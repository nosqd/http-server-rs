use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    vec,
};

use itertools::Itertools;

struct Request {
    full_url: String,
    url_parts: Vec<String>,
    method: String,
    http_version: String
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

    let mut headers: Vec<(String, String)> = vec![];

    loop {
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        if header == "\r\n" {
            break;
        }
        let header_parts = Vec::from_iter(header.split(": "));
        headers.push((
            header_parts[0].trim_end().into(),
            header_parts[1].trim_end().into(),
        ));
    }

    // todo parse post bodies

    return Ok(Request {
        full_url: url.to_string(),
        http_version: http_version.to_string(),
        method: method.to_string(),
        url_parts: url_parts.iter().map(|s| s.to_string()).collect_vec()
    })
}


enum HttpStatus {
    Ok,
    NotFound
}

impl HttpStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "200 OK",
            HttpStatus::NotFound => "404 Not Found"
        }
    }
}

struct Response {
    status: HttpStatus,
    headers: Vec<(String, String)>,
    body: String
}

impl Response {
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

fn handle_stream(stream: &TcpStream) {
    let req = read_request(stream).unwrap();

    if req.full_url == "/" {
        let resp = Response {
            body: "Hello, world".to_string(),
            status: HttpStatus::Ok,
            headers: vec![] 
        };
        _ = resp.write(stream).unwrap();
    } else if req.full_url.starts_with("/echo/") {
        let data_to_echo = req.url_parts.get(2).unwrap();
        let resp = Response {
            body: data_to_echo.to_string(),
            status: HttpStatus::Ok,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), data_to_echo.as_bytes().len().to_string()),
            ] 
        };
        _ = resp.write(stream).unwrap();
    } else {
        let resp = Response {
            body: "Not Found".to_string(),
            status: HttpStatus::NotFound,
            headers: vec![] 
        };
        _ = resp.write(stream).unwrap();
    }

    _ = stream.shutdown(std::net::Shutdown::Both).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("started on ::4221");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                _ = handle_stream(&stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
