use crate::statuscode::StatusCode;
use std::io::{Error, Write};
use std::process::{Command, Stdio};

pub struct Response {
    status: StatusCode,
    content_type: ContentType,
    accept_encoding: Option<AcceptEncoding>,
    body: Vec<u8>,
    connection_close: bool,
}

impl Response {
    pub fn new(
        status: StatusCode,
        accept_encoding: Option<AcceptEncoding>,
        content_type: ContentType,
        body: &str,
        connection_close: bool,
    ) -> Self {
        let compressed_body = match accept_encoding {
            Some(AcceptEncoding::Gzip) => match AcceptEncoding::compress_gzip(body) {
                Ok(compressed) => compressed,
                Err(e) => {
                    println!("err while compressing: {}", e);
                    body.as_bytes().to_vec()
                }
            },
            None => body.as_bytes().to_vec(),
        };

        Self {
            status,
            accept_encoding,
            content_type,
            body: compressed_body,
            connection_close,
        }
    }

    pub fn format_bytes(&self) -> Vec<u8> {
        let http_version = "HTTP/1.1";
        let content_length = self.body.len();
        let mut headers = format!(
            "{} {} {}\r\n",
            http_version,
            self.status.code(),
            self.status.reason_phrase()
        );

        if let Some(ref encoding) = self.accept_encoding {
            headers.push_str(&format!("Content-Encoding: {}\r\n", encoding.str()));
        }

        headers.push_str(&format!(
            "Content-Type: {}\r\nContent-Length: {}\r\n",
            self.content_type.str(),
            content_length
        ));

        if self.connection_close {
            headers.push_str("Connection: Close\r\n");
        }

        headers.push_str("\r\n");

        let mut response_bytes = headers.into_bytes();
        response_bytes.extend(&self.body);
        response_bytes
    }
}

pub enum ContentType {
    TextPlain,
    TextHtml,
    ApplicationOctetStream,
}

impl ContentType {
    pub fn str(&self) -> &str {
        match self {
            ContentType::TextPlain => "text/plain",
            ContentType::TextHtml => "text/html",
            ContentType::ApplicationOctetStream => "application/octet-stream",
        }
    }
}

pub enum AcceptEncoding {
    Gzip,
}

impl AcceptEncoding {
    pub fn str(&self) -> &str {
        match self {
            AcceptEncoding::Gzip => "gzip",
        }
    }

    pub fn compress_gzip(body: &str) -> Result<Vec<u8>, Error> {
        let mut child = Command::new("gzip")
            .arg("-c")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(body.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(Error::new(
                std::io::ErrorKind::Other,
                format!("err gzip processing: {:?}", output.status),
            ))
        }
    }
}
