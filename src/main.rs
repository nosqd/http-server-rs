use std::{
    default, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, time::Duration
};

fn handle_stream(mut stream: &TcpStream) {
    // old code

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut info_part = String::new();
    reader.read_line(&mut info_part).unwrap();
    let into_part_parts = Vec::from_iter(info_part.split(" "));
    let _method = into_part_parts.get(0).unwrap().trim_end();
    let url = into_part_parts.get(1).unwrap().trim_end();
    let _httpVersion = into_part_parts.get(2).unwrap().trim_end();

    println!(
        "[{}] sent {} request to {} url with {} http version",
        stream.peer_addr().unwrap(),
        _method,
        url,
        _httpVersion
    );

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
