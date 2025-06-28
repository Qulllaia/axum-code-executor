use axum::{response::IntoResponse, extract::{Path}};
use std::fs;
 
use std::path::Path as p;
use std::fs::File as f;

pub struct ExecuteController;
impl ExecuteController {

    pub fn new() -> Self {
        return Self
    }

    pub async fn execute_file(Path(id): Path<u32>) -> impl IntoResponse {
        return "File execution";
    }

    pub async fn create_file(Path(id): Path<u32>) -> impl IntoResponse {
        let dir_path = "static";  
 
        if !p::new(dir_path).exists() {
            let _ = fs::create_dir(dir_path); 
        }

        let _ = f::create(format!("./static/{id}.c"));

        return format!("file id {id}");
    }
}