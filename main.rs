use std::net::TcpListener;
use std::io::Write;
use std::io::Read;

fn main() {
    let port = 4221;
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&address).unwrap();
    println!("Server listening on port {}...", port);

    for stream in listener.incoming() {
         match stream {
            Ok(mut _stream) => {
                let mut buffer = [0; 256];
                let bytes = _stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..bytes]);
                // println!("{}", request);
                let request_parts = request.split("\r\n");
                let request_collection = request_parts.collect::<Vec<&str>>();

                let request_line = request_collection[0];
                let request_line_parts = request_line.split(" ");
                let request_line_parts_collection = request_line_parts.collect::<Vec<&str>>();
                let requested_path = request_line_parts_collection[1];
                // let host_header = request_collection[1];
                // let user_agent = request_collection[2];
                // let accept = request_collection[3];

                let http_version = "HTTP/1.1";

                let mut status_code = 200;
                let mut reason_phrase = "OK";

                let paths = vec!["/"];

                if !paths.contains(&requested_path) {
                    status_code = 404;
                    reason_phrase = "Not Found";
                }

                let response = format!(
                    "{} {} {}\r\n\r\n",
                    http_version,
                    status_code,
                    reason_phrase,
                );

                let _ = _stream.write(response.as_bytes());
            }
            Err(e) => {
                println!("err: {}", e);
            }
        }
    }
}
