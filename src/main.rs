use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

fn handle_stream(mut stream: &TcpStream) {
    let response_data = "HTTP/1.1 200 OK\r\n\r\n";

    let mut response_bytes: Vec<u8> = vec![];
    response_bytes.extend(response_data.as_bytes());
    let _ = stream.write_all(&response_bytes);
    //_ = stream.shutdown(std::net::Shutdown::Both).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("started on ::4221");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                _ = handle_stream(&stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
