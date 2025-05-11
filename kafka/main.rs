mod threadpool;

use crate::threadpool::ThreadPool;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9092").unwrap();
    println!("listening on 9092..");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                pool.execute(move || {
                    println!("accepted new connection");
                    handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let header = parse_header(&mut stream);
        let response = create_response(&header);
        send_response(&mut stream, &response);
    }
}

struct Request {
    message_size: u32,
    request_api_key: i16,
    request_api_version: i16,
    correlation_id: i32,
}

fn parse_header(stream: &mut TcpStream) -> Request {
    const LEN: usize = 4 + 2 + 2 + 4;
    let mut buffer = [0u8; LEN];

    if let Err(e) = stream.read_exact(&mut buffer) {
        println!("err: failed read the header: {}", e);
    }

    let message_size = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let request_api_key = i16::from_be_bytes([buffer[4], buffer[5]]);
    let request_api_version = i16::from_be_bytes([buffer[6], buffer[7]]);
    let correlation_id = i32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);

    let remaining = message_size as usize - 8;
    let mut discard = vec![0u8; remaining];
    if let Err(e) = stream.read_exact(&mut discard) {
        println!("err: failed to discard remaining request: {}", e);
    };

    Request {
        message_size,
        request_api_key,
        request_api_version,
        correlation_id,
    }
}

fn send_response(stream: &mut TcpStream, response: &[u8]) {
    if let Err(e) = stream.write_all(response) {
        println!("err: failed to send response: {}", e);
    }
}

struct ApiKey {
    api_key: i16,
    min_api_version: i16,
    max_api_version: i16,
}

fn create_response(header: &Request) -> Vec<u8> {
    const ERROR: i16 = 35;
    const NO_ERROR: i16 = 0;
    const TAG_BUFFER_LEN: i8 = 0;
    const THROTTLE_TIME_MS: i32 = 0;

    let error_code: i16 = if header.request_api_version < 0 || header.request_api_version > 4 {
        ERROR
    } else {
        NO_ERROR
    };

    let api_keys: Vec<ApiKey> = vec![ApiKey {
        api_key: 18,
        min_api_version: 0,
        max_api_version: 4,
    }];

    let size = 12 + (api_keys.len() * 7) as i32;
    let api_key_count = (api_keys.len() + 1) as i8;

    let mut response: Vec<u8> = Vec::new();

    response.extend_from_slice(&size.to_be_bytes());
    response.extend_from_slice(&header.correlation_id.to_be_bytes());
    response.extend_from_slice(&error_code.to_be_bytes());
    response.extend_from_slice(&api_key_count.to_be_bytes());

    for key in &api_keys {
        response.extend_from_slice(&key.api_key.to_be_bytes());
        response.extend_from_slice(&key.min_api_version.to_be_bytes());
        response.extend_from_slice(&key.max_api_version.to_be_bytes());
        response.extend_from_slice(&TAG_BUFFER_LEN.to_be_bytes());
    }

    response.extend_from_slice(&THROTTLE_TIME_MS.to_be_bytes());
    response.push(0x00);

    response
}
