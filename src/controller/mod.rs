mod types;
use axum::{extract::{FromRequest, Path, Request}, response::IntoResponse, Error, Json};
use serde_json::json;
use std::fs;
use types::CreateCodeRequest;
use std::path::Path as p;
use std::fs::File as f;
use uuid::Uuid;
use std::io::{Write};
use tokio::process::Command;

static DIR_PATH: &'static str = "static";  
#[derive(Debug)]
pub struct ExecuteController;
impl ExecuteController {

    pub fn new() -> Self {
        return Self
    }

    // Это доделать надо бы блин
    pub async fn execute_file(Path(id): Path<String>) -> impl IntoResponse {
        let path = format!("../{}/{}.c", DIR_PATH.to_string(), &id);
        println!("{}",format!("gcc {} -o output",  &path));

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("gcc {} -o output",  &path))
            .output()
            .await;
        println!("{:?}", output);
        return "File execution";
    }
    pub async fn create_file(
        Json(file_data): Json<CreateCodeRequest>
    ) -> Json<serde_json::Value> {

        let file_id: Uuid = Uuid::new_v4();
        let _ = Self::file_generator(file_data.code.unwrap(), file_id).await;
        return Json(serde_json::json!(
            {
                "result":"done",
            }
        ));
    }

    pub async fn update_file(Json(file_data): Json<CreateCodeRequest>) -> Json<serde_json::Value> {
        let file_id: &String = &file_data.file_name.unwrap();
        let searching_file_result = Self::check_if_file_exists(file_id).await;
        match searching_file_result {
            Ok(_) => {
                 let _ = Self::file_generator(file_data.code.unwrap(), Uuid::parse_str(file_id.as_str()).unwrap()).await;
                return Json(serde_json::json!(
                {
                    "result":"done",
                }
            ))
            },
            Err(e) =>Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            )),
        }
    }

    async fn file_generator(code: String, uid: Uuid) -> Result<(), std::io::Error> {
        let dir_path = DIR_PATH;

        if !p::new(dir_path).exists() {
            let _ = fs::create_dir(dir_path); 
        }

        let mut file = f::create(format!("./static/{uid}.c"))?;
        file.write_all(code.as_bytes())?;
        Ok(())
    }

    async fn check_if_file_exists(file_name: &str) -> Result<(), std::io::Error> {
        let dir = fs::read_dir(DIR_PATH)?;   
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            if file_name == path.file_stem().unwrap().to_str().unwrap() {
                return Ok(());
            }
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));

    }
}