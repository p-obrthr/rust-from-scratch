use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

struct Header {
    message_size: u32,
    request_api_key: i16,
    request_api_version: i16,
    correlation_id: i32,
}

struct Response {
    message_size: [u8; 4],
    correlation_id: [u8; 4],
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9092").unwrap();
    println!("listening on 9092..");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let header = parse_header(stream);
    let response = create_response(header.correlation_id);
    send_response(stream, &response);
}

fn create_response(correlation_id: i32) -> Response {
    let message_size: i32 = 0;
    Response {
        message_size: message_size.to_be_bytes(),
        correlation_id: correlation_id.to_be_bytes(),
    }
}

fn parse_header(stream: &mut TcpStream) -> Header {
    const LEN: usize = 4 + 2 + 2 + 4;
    let mut buffer = [0u8; LEN];

    match stream.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(e) => println!("err: failed read the header: {}", e),
    }

    // 4
    let message_size = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    // 2
    let request_api_key = i16::from_be_bytes([buffer[4], buffer[5]]);
    // 2
    let request_api_version = i16::from_be_bytes([buffer[6], buffer[7]]);
    // 4
    let correlation_id = i32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    // --> 8 bytes after message size

    let remaining = message_size as usize - 8;
    let mut discard = vec![0u8; remaining];
    match stream.read_exact(&mut discard) {
        Ok(_) => {}
        Err(e) => println!("err: failed to discard remaining request: {}", e),
    };

    println!("message_size: {}", message_size);
    println!("request_api_key: {}", request_api_key);
    println!("request_api_version: {}", request_api_version);
    println!("correlation_id: {}", correlation_id);

    Header {
        message_size,
        request_api_key,
        request_api_version,
        correlation_id,
    }
}

fn send_response(stream: &mut TcpStream, response: &Response) {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&response.message_size);
    buffer.extend_from_slice(&response.correlation_id);
    match stream.write(&buffer) {
        Ok(_) => {}
        Err(e) => println!("err: failed to send response: {}", e),
    }
}
