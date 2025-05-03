mod response;
mod statuscode;
mod threadpool;

use crate::response::{AcceptEncoding, ContentType, Response};
use crate::statuscode::StatusCode;
use crate::threadpool::ThreadPool;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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

    loop {
        let mut buffer = [0; 256];
        let bytes = match _stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                println!("err while reading bytes: {}", e);
                break;
            }
        };

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
        let mut accept_encoding: Vec<String> = Vec::new();
        let mut connection = String::new();

        parse_headers(
            &header_parts,
            &mut user_agent_kind,
            &mut host,
            &mut accept,
            &mut req_content_type,
            &mut req_content_len,
            &mut accept_encoding,
            &mut connection,
        );

        let connection_close: bool = connection.to_lowercase() == "close";

        let response = process_request(
            method,
            req_path_edited,
            &user_agent_kind,
            req_body,
            directory,
            &accept_encoding,
            connection_close,
        );

        let response_bytes = response.format_bytes();
        let _ = _stream.write(&response_bytes);

        if connection_close {
            break;
        }
    }
}

fn parse_headers(
    header_parts: &[&str],
    user_agent_kind: &mut String,
    host: &mut String,
    accept: &mut String,
    req_content_type: &mut String,
    req_content_len: &mut usize,
    accept_encoding: &mut Vec<String>,
    connection: &mut String,
) {
    for part in header_parts {
        let headers = part.split(": ").collect::<Vec<&str>>();
        match headers[0].to_lowercase().as_str() {
            "host" => {
                *host = headers[1].to_string();
            }
            "user-agent" => {
                *user_agent_kind = headers[1].to_string();
            }
            "accept" => {
                *accept = headers[1].to_string();
            }
            "content-type" => {
                *req_content_type = headers[1].to_string();
            }
            "content-length" => {
                *req_content_len = headers[1].parse::<usize>().unwrap();
            }
            "accept-encoding" => {
                *accept_encoding = headers[1]
                    .split(", ")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
            }
            "connection" => {
                *connection = headers[1].to_string();
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
    accept_encoding: &Vec<String>,
    connection_close: bool,
) -> Response {
    let req_path_parts = path.split("/").collect::<Vec<&str>>();

    let mut parsed_encoding = None;
    for encoding in accept_encoding {
        if encoding == "gzip" {
            parsed_encoding = Some(AcceptEncoding::Gzip);
            break;
        }
    }

    if req_path_parts.is_empty() {
        return Response::new(
            StatusCode::Ok,
            parsed_encoding,
            ContentType::TextPlain,
            "",
            connection_close,
        );
    }

    match req_path_parts[0] {
        "echo" => {
            if req_path_parts.len() > 1 {
                Response::new(
                    StatusCode::Ok,
                    parsed_encoding,
                    ContentType::TextPlain,
                    req_path_parts[1],
                    connection_close,
                )
            } else {
                Response::new(
                    StatusCode::NotFound,
                    parsed_encoding,
                    ContentType::TextPlain,
                    "",
                    connection_close,
                )
            }
        }
        "" => Response::new(
            StatusCode::Ok,
            parsed_encoding,
            ContentType::TextPlain,
            "",
            connection_close,
        ),
        "user-agent" => Response::new(
            StatusCode::Ok,
            parsed_encoding,
            ContentType::TextPlain,
            user_agent,
            connection_close,
        ),
        "files" => {
            if req_path_parts.len() < 2 {
                Response::new(
                    StatusCode::NotFound,
                    parsed_encoding,
                    ContentType::TextPlain,
                    "",
                    connection_close,
                )
            } else {
                let file_path = format!("{}/{}", directory, req_path_parts[1]);
                if method == "POST" {
                    let result = fs::write(&file_path, body);
                    match result {
                        Ok(_result) => Response::new(
                            StatusCode::Created,
                            parsed_encoding,
                            ContentType::TextPlain,
                            "",
                            connection_close,
                        ),
                        Err(e) => {
                            println!("err: {e}");
                            Response::new(
                                StatusCode::InternalServerError,
                                parsed_encoding,
                                ContentType::TextPlain,
                                "",
                                connection_close,
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
                            Response::new(
                                StatusCode::Ok,
                                parsed_encoding,
                                content_type,
                                &body,
                                connection_close,
                            )
                        }
                        Err(e) => {
                            println!("err: {e}");
                            Response::new(
                                StatusCode::NotFound,
                                parsed_encoding,
                                ContentType::TextPlain,
                                "",
                                connection_close,
                            )
                        }
                    }
                }
            }
        }
        _ => Response::new(
            StatusCode::NotFound,
            parsed_encoding,
            ContentType::TextPlain,
            "",
            connection_close,
        ),
    }
}
