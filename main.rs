mod lib;
mod response;
mod statuscode;

use crate::lib::ThreadPool;
use crate::response::{ContentType, Response};
use crate::statuscode::StatusCode;
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

    let mut user_agent_kind = String::new();
    let mut host = String::new();
    let mut accept = String::new();
    let mut req_content_type = String::new();
    let mut req_content_len: usize = 0;

    parse_headers(
        &header_parts,
        &mut user_agent_kind,
        &mut host,
        &mut accept,
        &mut req_content_type,
        &mut req_content_len,
    );

    let response = process_request(
        method,
        req_path_edited,
        &user_agent_kind,
        req_body,
        directory,
    );

    let _ = _stream.write(response.format_string().as_bytes());
}

fn parse_headers(
    header_parts: &[&str],
    user_agent_kind: &mut String,
    host: &mut String,
    accept: &mut String,
    req_content_type: &mut String,
    req_content_len: &mut usize,
) {
    for part in header_parts {
        let headers = part.split(" ").collect::<Vec<&str>>();
        match headers[0].to_lowercase().as_str() {
            "host:" => {
                *host = headers[1].to_string();
            }
            "user-agent:" => {
                *user_agent_kind = headers[1].to_string();
            }
            "accept:" => {
                *accept = headers[1].to_string();
            }
            "content-type:" => {
                *req_content_type = headers[1].to_string();
            }
            "content-length:" => {
                *req_content_len = headers[1].parse::<usize>().unwrap();
            }
            _ => {}
        }
    }
}

fn process_request(
    method: &str,
    path: &str,
    user_agent: &str,
    body: &str,
    directory: &str,
) -> Response {
    let req_path_parts = path.split("/").collect::<Vec<&str>>();

    if req_path_parts.is_empty() {
        return Response::new(StatusCode::Ok, ContentType::TextPlain, "");
    }

    match req_path_parts[0] {
        "echo" => {
            if req_path_parts.len() > 1 {
                Response::new(StatusCode::Ok, ContentType::TextPlain, req_path_parts[1])
            } else {
                Response::new(StatusCode::NotFound, ContentType::TextPlain, "")
            }
        }
        "" => Response::new(StatusCode::Ok, ContentType::TextPlain, ""),
        "user-agent" => Response::new(StatusCode::Ok, ContentType::TextPlain, user_agent),
        "files" => {
            if req_path_parts.len() < 2 {
                Response::new(StatusCode::NotFound, ContentType::TextPlain, "")
            } else {
                let file_path = format!("{}/{}", directory, req_path_parts[1]);
                if method == "POST" {
                    let result = fs::write(&file_path, body);
                    match result {
                        Ok(_result) => {
                            Response::new(StatusCode::Created, ContentType::TextPlain, "")
                        }
                        Err(e) => {
                            println!("err: {e}");
                            Response::new(
                                StatusCode::InternalServerError,
                                ContentType::TextPlain,
                                "",
                            )
                        }
                    }
                } else {
                    let contents = fs::read(&file_path);
                    match contents {
                        Ok(bytes) => {
                            let content_type = if req_path_parts[1].ends_with(".html") {
                                ContentType::TextHtml
                            } else {
                                ContentType::ApplicationOctetStream
                            };

                            let body = String::from_utf8_lossy(&bytes)
                                .trim_end_matches('\n')
                                .to_string();
                            Response::new(StatusCode::Ok, content_type, &body)
                        }
                        Err(e) => {
                            println!("err: {e}");
                            Response::new(StatusCode::NotFound, ContentType::TextPlain, "")
                        }
                    }
                }
            }
        }
        _ => Response::new(StatusCode::NotFound, ContentType::TextPlain, ""),
    }
}
