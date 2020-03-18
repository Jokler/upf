use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone)]
pub struct UploaderTemplate {
    pub method: Method,
    pub request_url: String,
    pub data: DataType,
    pub form: HashMap<String, String>,
    pub file_form: Option<String>,
    pub headers: HashMap<String, String>,
    // TODO add these
    //pub url_params: HashMap<String, String>,
    pub regex: Option<String>,
    pub url: String,
    pub additional_urls: HashMap<String, String>,
    pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum DataType {
    NoBody,
    Plain,
    Multipart,
    FormUrlEncoded,
    Json,
    Xml,
}

impl UploaderTemplate {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TemplateError> {
        let file = File::open(path).map_err(TemplateError::OpenFile)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .map_err(TemplateError::ReadFile)?;

        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Failed to open file")]
    OpenFile(io::Error),
    #[error("Failed to read file")]
    ReadFile(io::Error),
    #[error("Failed to parse toml")]
    ParseToml(#[from] toml::de::Error),
}
