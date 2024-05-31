use clap::Parser;
use itertools::Itertools;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, thread, vec};
use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

struct Request {
    full_url: String,
    url_parts: Vec<String>,
    headers: HashMap<String, String>,
    method: String,
    http_version: String,
    body: Vec<u8>,
}

fn read_request(stream: &TcpStream) -> io::Result<Request> {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut info_part = String::new();
    reader.read_line(&mut info_part).unwrap();
    let into_part_parts = Vec::from_iter(info_part.split(" "));
    let method = into_part_parts.get(0).unwrap().trim_end();
    let url = into_part_parts.get(1).unwrap().trim_end();
    let url_parts = Vec::from_iter(url.split("/"));
    let http_version = into_part_parts.get(2).unwrap().trim_end();

    let mut headers: HashMap<String, String> = HashMap::new();

    loop {
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        if header == "\r\n" {
            break;
        }
        let header_parts = Vec::from_iter(header.split(": "));
        headers.insert(
            header_parts[0].trim_end().into(),
            header_parts[1].trim_end().into(),
        );
    }

    // body
    let mut d = Request {
        full_url: url.to_string(),
        http_version: http_version.to_string(),
        method: method.to_string(),
        url_parts: url_parts.iter().map(|s| s.to_string()).collect_vec(),
        headers: headers.clone(),
        body: vec![],
    };

    if headers.contains_key("Content-Type") {
        let length: usize = headers
            .get("Content-Length")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        if length != 0 {
            let mut body: Vec<u8> = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            d.body = body.clone();
        }
    }

    return Ok(d);
}

enum HttpStatus {
    Ok,
    NotFound,
    InternalServerError,
}

impl HttpStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "200 OK",
            HttpStatus::NotFound => "404 Not Found",
            HttpStatus::InternalServerError => "500 Internal Server Error",
        }
    }
}

struct Response {
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    fn add_content_headers(&mut self, content_type: &str) {
        self.headers
            .insert("Content-Type".to_string(), content_type.to_string());
        self.headers.insert(
            "Content-Length".to_string(),
            self.body.as_bytes().len().to_string(),
        );
    }

    fn write(&self, mut stream: &TcpStream) -> io::Result<usize> {
        // Status line
        // HTTP/1.1 200 OK
        // \r\n                          // CRLF that marks the end of the status line

        // Headers
        // Content-Type: text/plain\r\n  // Header that specifies the format of the response body
        // Content-Length: 3\r\n         // Header that specifies the size of the response body, in bytes
        // \r\n                          // CRLF that marks the end of the headers

        // Response body
        // abc                           // The string from the request

        let mut response = String::new();

        response += "HTTP/1.1 ";
        response += self.status.as_str();
        response += "\r\n";

        for header in self.headers.clone() {
            response += header.0.as_str();
            response += ": ";
            response += header.1.as_str();
            response += "\r\n";
        }
        response += "\r\n";
        response += self.body.as_str();
        stream.write(response.as_bytes())
    }
}

fn handle_stream(args: Args, stream: &TcpStream) {
    let req = read_request(stream).unwrap();

    if req.full_url == "/" {
        let mut resp = Response {
            body: "Hello, world".to_string(),
            status: HttpStatus::Ok,
            headers: HashMap::new(),
        };
        resp.add_content_headers("text/plain");
        _ = resp.write(stream).unwrap();
    }
    if req.full_url == "/user-agent" {
        let mut resp = Response {
            body: req.headers.get("User-Agent").unwrap().to_string(),
            status: HttpStatus::Ok,
            headers: HashMap::new(),
        };
        resp.add_content_headers("text/plain");
        _ = resp.write(stream).unwrap();
    } else if req.full_url.starts_with("/echo/") {
        let data_to_echo = req.url_parts.get(2).unwrap();
        let mut resp = Response {
            body: data_to_echo.to_string(),
            status: HttpStatus::Ok,
            headers: HashMap::new(),
        };
        resp.add_content_headers("text/plain");
        _ = resp.write(stream).unwrap();
    } else if req.full_url.starts_with("/files/") {
        let file_path = req.url_parts[2..].to_vec();
        let mut path: PathBuf = [args.directory.unwrap_or(".".to_string())].iter().collect();
        path.extend(file_path.iter().map(|v| v.as_str()));

        println!("{}", path.as_os_str().to_str().unwrap());
        if req.method == "GET" {
            match std::fs::read_to_string(path) {
                Ok(data) => {
                    let mut resp = Response {
                        body: data,
                        status: HttpStatus::Ok,
                        headers: HashMap::new(),
                    };
                    resp.add_content_headers("application/octet-stream");
                    _ = resp.write(stream).unwrap();
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        let mut resp = Response {
                            body: "Not Found".to_string(),
                            status: HttpStatus::NotFound,
                            headers: HashMap::new(),
                        };
                        resp.add_content_headers("text/plain");
                        _ = resp.write(stream).unwrap();
                    } else {
                        let mut resp = Response {
                            body: "Internal Server Error".to_string(),
                            status: HttpStatus::InternalServerError,
                            headers: HashMap::new(),
                        };
                        resp.add_content_headers("text/plain");
                        _ = resp.write(stream).unwrap();
                    }
                }
            }
        } else if req.method == "POST" {
            fs::write(path, req.body).unwrap_or_else(|_| {
                let mut resp = Response {
                    body: "Internal Server Error".to_string(),
                    status: HttpStatus::InternalServerError,
                    headers: HashMap::new(),
                };
                resp.add_content_headers("text/plain");
                _ = resp.write(stream).unwrap();
            });
            let mut resp = Response {
                body: "ok".to_string(),
                status: HttpStatus::Ok,
                headers: HashMap::new(),
            };
            resp.add_content_headers("text/plain");
            _ = resp.write(stream).unwrap();
        }
    } else {
        let mut resp = Response {
            body: "Not Found".to_string(),
            status: HttpStatus::NotFound,
            headers: HashMap::new(),
        };
        resp.add_content_headers("text/plain");
        _ = resp.write(stream).unwrap();
    }

    _ = stream.shutdown(std::net::Shutdown::Both).unwrap();
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    directory: Option<String>,
}

fn main() {
    let args = Args::parse();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("started on ::4221");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let a = args.clone();
                thread::spawn(move || {
                    _ = handle_stream(a, &stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
