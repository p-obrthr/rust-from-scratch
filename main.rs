mod lib;
use lib::ThreadPool;
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
    let mut buffer = [0; 256];
    let bytes = _stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..bytes]);
    // println!("{}", request);
    let request_parts = request.split("\r\n").collect::<Vec<&str>>();

    let request_line = request_parts[0];
    //println!("{request_line}");
    let request_line_parts = request_line.split(" ").collect::<Vec<&str>>();
    let requested_path = request_line_parts[1];
    let requested_path_edited = &requested_path[1..];

    // let host_header = request_parts[1];
    let user_agent = request_parts[2];
    let user_agent_parts = user_agent.split(" ").collect::<Vec<&str>>();
    let mut user_agent_kind = String::new();
    if user_agent_parts.len() > 1 {
        user_agent_kind = user_agent_parts[1].to_string();
    }
    println!("{user_agent_kind}");

    // let accept = request_collection[3];

    // println!("{requested_path}");
    // let paths = ["/", "", "echo"];
    let mut response_body = String::new();
    let mut status_code = 200;
    let mut reason_phrase = "OK";

    let requested_path_parts = requested_path_edited.split("/").collect::<Vec<&str>>();
    if !requested_path_parts.is_empty() {
        println!("{}", requested_path_parts[0]);

        match requested_path_parts[0] {
            "echo" => {
                if requested_path_parts.len() > 1 {
                    response_body.push_str(requested_path_parts[1]);
                }
            }
            "" => {}
            "user-agent" => {
                response_body.push_str(&user_agent_kind);
            }
            _ => {
                status_code = 404;
                reason_phrase = "Not Found";
            }
        }
    }
    // println!("{}", requested_path_parts[0]);
    let http_version = "HTTP/1.1";

    let content_type = "Content-Type: text/plain";
    let content_length = if response_body.is_empty() {
        "Content-Length: 0".to_string()
    } else {
        format!("Content-Length: {}", response_body.len())
    };

    let response = format!(
        "{} {} {}\r\n{}\r\n{}\r\n\r\n{}",
        http_version, status_code, reason_phrase, content_type, content_length, response_body
    );

    let _ = _stream.write(response.as_bytes());
}
