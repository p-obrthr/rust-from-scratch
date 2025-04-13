mod lib;
use lib::ThreadPool;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    let port = 4221;
    let address = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&address).unwrap();
    println!("Server listening on port {port}...");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                pool.execute(|| {
                    handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("err: {}", e);
            }
        }
    }
}

fn handle_connection(mut _stream: TcpStream) {
    let args: Vec<String> = std::env::args().collect();
    let directory = if args.len() >= 3 && args[1] == "--directory" {
        &args[2]
    } else {
        "."
    };

    let mut buffer = [0; 256];
    let bytes = _stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..bytes]);
    // println!("{}", request);

    let mut main_parts = request.splitn(2, "\r\n\r\n");
    let headers = main_parts.next().unwrap_or("");
    let req_body = main_parts.next().unwrap_or("");

    let header_parts = headers.split("\r\n").collect::<Vec<&str>>();
    let req_line = header_parts[0].split(" ").collect::<Vec<&str>>();

    let method = req_line[0];
    let req_path = req_line[1];
    let req_path_edited = &req_path[1..];

    // let host_header = request_parts[1];
    let mut user_agent_kind = String::new();
    let mut host = String::new();
    let mut accept = String::new();
    let mut req_content_type = String::new();
    let mut req_content_len: usize = 0;

    for part in &header_parts {
        let headers = part.split(" ").collect::<Vec<&str>>();
        match headers[0].to_lowercase().as_str() {
            "host:" => {
                host = headers[1].to_string();
            }
            "user-agent:" => {
                user_agent_kind = headers[1].to_string();
            }
            "accept:" => {
                accept = headers[1].to_string();
            }
            "content-type:" => {
                req_content_type = headers[1].to_string();
            }
            "content-length:" => {
                req_content_len = headers[1].parse::<usize>().unwrap();
            }
            _ => {}
        }
    }

    let mut resp_body = String::new();
    let mut status_code = 200;
    let mut reason_phrase = "OK";

    let mut resp_content_kind = "text/plain";

    let req_path_parts = req_path_edited.split("/").collect::<Vec<&str>>();
    if !req_path_parts.is_empty() {
        //println!("{}", requested_path_parts[0]);

        match req_path_parts[0] {
            "echo" => {
                if req_path_parts.len() > 1 {
                    resp_body.push_str(req_path_parts[1]);
                }
            }
            "" => {}
            "user-agent" => {
                resp_body.push_str(&user_agent_kind);
            }
            "files" => {
                if req_path_parts.len() > 1 {
                    let file_path = format!("{}/{}", directory, req_path_parts[1]);
                    if method == "POST" {
                        let result = fs::write(&file_path, req_body);
                        match result {
                            Ok(_result) => {
                                status_code = 201;
                                reason_phrase = "Created";
                            }
                            Err(e) => {
                                status_code = 500;
                                reason_phrase = "Internal Server Error";
                                println!("err: {e}");
                            }
                        }
                    } //println!("In file {file_path}");
                    let contents: Result<Vec<u8>, std::io::Error> = fs::read(file_path);
                    match contents {
                        Ok(mut bytes) => {
                            if bytes.ends_with(&[b'\n']) {
                                bytes.pop();
                            }
                            //println!("bytes:\n{:?}", bytes);
                            //println!("{}", bytes.len());

                            resp_content_kind = if req_path_parts[1].ends_with(".html") {
                                "text/html"
                            } else {
                                "application/octet-stream"
                            };
                            resp_body.push_str(&String::from_utf8_lossy(&bytes));
                        }
                        Err(e) => {
                            status_code = 404;
                            reason_phrase = "Not Found";
                            println!("err: {e}");
                        }
                    }
                }
            }
            _ => {
                status_code = 404;
                reason_phrase = "Not Found";
            }
        }
    }
    // println!("{}", requested_path_parts[0]);
    let http_version = "HTTP/1.1";

    let resp_content_len = if resp_body.is_empty() {
        0
    } else {
        resp_body.len()
    };

    let response = format!(
        "{} {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        http_version, status_code, reason_phrase, resp_content_kind, resp_content_len, resp_body
    );

    let _ = _stream.write(response.as_bytes());
}
