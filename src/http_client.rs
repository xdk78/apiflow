use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ureq::serde_json;

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
            response_body: Ok(String::new()),
        }
    }
}

pub struct HTTPClient {
    pub http_method: HTTPMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub response_body: Result<String, String>,
}

impl HTTPClient {
    pub fn send_request(&mut self, data: Option<impl serde::Serialize>) {
        let mut request = ureq::request(&self.http_method.to_string(), &self.url);

        for (key, value) in &self.headers {
            request = request.set(key, value);
        }

        let response = match data {
            // TODO: handle error
            Some(data) => request.send_json(serde_json::to_value(data).unwrap()),
            None => request.call(),
        };

        match response {
            Ok(response) => {
                if response.status() == 200 {
                    let body = response.into_string();
                    match body {
                        Ok(body) => {
                            self.response_body = Ok(body);
                        }
                        Err(err) => {
                            self.response_body = Err(err.to_string());
                        }
                    }
                } else {
                    self.response_body = Err(format!("HTTP Error: {}", response.status()));
                }
            }
            Err(err) => {
                self.response_body = Err(err.to_string());
            }
        }
    }
}
