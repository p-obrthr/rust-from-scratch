use crate::statuscode::StatusCode;

pub struct Response {
    status: StatusCode,
    content_type: ContentType,
    body: String,
}

impl Response {
    pub fn new(status: StatusCode, content_type: ContentType, body: &str) -> Self {
        Self {
            status,
            content_type,
            body: body.to_string(),
        }
    }

    pub fn format_string(&self) -> String {
        let http_version = "HTTP/1.1";
        let content_length = self.body.len();

        format!(
            "{} {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            http_version,
            self.status.code(),
            self.status.reason_phrase(),
            self.content_type.str(),
            content_length,
            self.body
        )
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
