use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum HTTPMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HTTPMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPMethod::Get => write!(f, "GET"),
            HTTPMethod::Post => write!(f, "POST"),
            HTTPMethod::Put => write!(f, "PUT"),
            HTTPMethod::Delete => write!(f, "DELETE"),
            HTTPMethod::Patch => write!(f, "PATCH"),
            HTTPMethod::Head => write!(f, "HEAD"),
            HTTPMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

pub struct HTTPClientBuilder {
    http_method: HTTPMethod,
    url: String,
    headers: HashMap<String, String>,
}

impl HTTPClientBuilder {
    pub fn new() -> Self {
        Self {
            http_method: HTTPMethod::Get,
            url: String::new(),
            headers: HashMap::new(),
        }
    }

    pub fn with_http_method(self, http_method: HTTPMethod) -> Self {
        Self {
            http_method,
            ..self
        }
    }

    pub fn with_url(self, url: String) -> Self {
        Self { url, ..self }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn build(self) -> HTTPClient {
        HTTPClient {
            http_method: self.http_method,
            url: self.url,
            headers: self.headers,
            response: ureq::Response::new(200, "OK", "body"),
        }
    }
}

#[derive(Debug)]
pub struct HTTPClient {
    http_method: HTTPMethod,
    url: String,
    headers: HashMap<String, String>,
    pub response: Result<ureq::Response, ureq::Error>,
}

impl HTTPClient {
    pub fn send_request<T: serde::Serialize>(&mut self, data: Option<T>) {
        let mut request = ureq::request(&self.http_method.to_string(), &self.url);

        for (key, value) in &self.headers {
            request = request.set(key, value);
        }

        let response = match data {
            Some(data) => {
                let data = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
                request.send_json(data)
            }
            None => request.call(),
        };

        match response {
            Ok(response) => {
                self.response = Ok(response);
            }
            Err(err) => {
                self.response = Err(err);
            }
        }
    }
}
