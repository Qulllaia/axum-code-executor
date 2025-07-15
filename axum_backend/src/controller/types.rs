use serde::de::{Deserialize};

#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct CreateCodeRequest {
    pub code: Option<String>,
    pub file_name: Option<String>,
}