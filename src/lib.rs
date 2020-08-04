use std::collections::HashMap;

use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, InvalidHeaderName, InvalidHeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use thiserror::Error;

mod template;
pub use template::*;

pub async fn upload(
    template: UploaderTemplate,
    data: Vec<u8>,
    file_name: Option<String>,
    debug: bool,
) -> Result<UploadResponse, UploadError> {
    let client = Client::new();
    let mut request = client.request(template.method.into(), &template.request_url);

    let mut headers = HeaderMap::new();
    for (key, value) in &template.headers {
        let name = HeaderName::from_bytes(key.as_bytes())?;
        headers.insert(name, value.parse()?);
    }
    request = request.headers(headers);

    match template.data {
        DataType::NoBody => (),
        DataType::Multipart => {
            let mut form = Form::new();
            if let Some(file_form) = &template.file_form {
                let mut part = Part::bytes(data);
                if let Some(file_name) = file_name {
                    part = part.file_name(file_name);
                }
                form = form.part(file_form.clone(), part);
            }
            for (key, value) in &template.form {
                form = form.text(key.to_owned(), value.to_owned());
            }
            request = request.multipart(form);
        }
        _ => unimplemented!(),
    }

    let response = request.send().await.map_err(UploadError::Request)?;

    let status = response.status();
    let body = response.text().await.map_err(UploadError::ParseResponse)?;

    if !status.is_success() {
        return Err(UploadError::Response(status, body));
    }

    if debug {
        println!("Response body: {}", body);
    }

    UploadResponse::find(&body, template)
}

pub struct UploadResponse {
    pub url: String,
    pub additional_urls: HashMap<String, String>,
}

impl UploadResponse {
    pub fn find(body: &str, template: UploaderTemplate) -> Result<Self, UploadError> {
        let mut url = template.url;
        let mut additional_urls = template.additional_urls;

        if let Some(regex) = template.regex {
            let re = Regex::new(&regex)?;
            let captures = re.captures(body).ok_or(UploadError::RegexNotFound(regex))?;

            for (i, cap) in captures.iter().enumerate() {
                if let Some(cap) = cap {
                    url = url.replace(&format!("$regex:{}$", i), cap.as_str());
                    for (_, url) in &mut additional_urls {
                        *url = url.replace(&format!("$regex:{}$", i), cap.as_str());
                    }
                } else {
                    eprintln!("Regex capture {} was not found", i);
                }
            }
        }

        Ok(Self {
            url,
            additional_urls,
        })
    }
}

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("Invalid header name")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    #[error("Invalid header name")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("Failed to send request")]
    Request(#[source] reqwest::Error),
    #[error("Received \"{0}\" from server: {1}")]
    Response(reqwest::StatusCode, String),
    #[error("Failed to parse response")]
    ParseResponse(#[source] reqwest::Error),
    #[error("Failed to parse regex")]
    Regex(#[from] regex::Error),
    #[error("Regex failed to capture anything: {0}")]
    RegexNotFound(String),
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> reqwest::Method {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
        }
    }
}
