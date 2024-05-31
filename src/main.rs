use std::{
    io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, vec
};

fn handle_stream(mut stream: &TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut info_part = String::new();
    reader.read_line(&mut info_part).unwrap();
    let into_part_parts = Vec::from_iter(info_part.split(" "));
    let _method = into_part_parts.get(0).unwrap().trim_end();
    let url = into_part_parts.get(1).unwrap().trim_end();
    let _http_version = into_part_parts.get(2).unwrap().trim_end();

    println!(
        "Recved {} request from {} to {} url with {} http version",
        _method,
        stream.peer_addr().unwrap(),
        url,
        _http_version
    );

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
            header_parts[1].trim_end().into()
        ));
    }

    println!("Headers:");
    for header in headers {
        println!("\t {} = {}", header.0, header.1);
    }

    const STATUS_OK: &str = "HTTP/1.1 200 OK\r\n\r\n";
    const STATUS_NF: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

    let mut response_bytes: Vec<u8> = vec![];
    response_bytes.extend(
        match url {
            "/" => STATUS_OK.as_bytes(),
            _ =>  STATUS_NF.as_bytes()
        }
    );
    let _ = stream.write_all(&response_bytes);
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
