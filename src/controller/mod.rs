mod types;
use axum::{extract::{FromRequest, Path, Request}, response::IntoResponse, Error, Json};
use serde_json::json;
use std::fs;
use types::CreateCodeRequest;
use std::path::Path as p;
use std::fs::File as f;
use uuid::Uuid;
use std::io::{Write};

static DIR_PATH: &'static str = "static";  
#[derive(Debug)]
pub struct ExecuteController;
impl ExecuteController {

    pub fn new() -> Self {
        return Self
    }

    pub async fn execute_file(Path(id): Path<u32>) -> impl IntoResponse {
        return "File execution";
    }
    pub async fn create_file(
        Json(file_data): Json<CreateCodeRequest>
    ) -> Json<serde_json::Value> {

        println!("{:?}", file_data);

        let file_id: Uuid = Uuid::new_v4();
        let _ = Self::file_generator(file_data.code.unwrap(), file_id).await;
        Self::check_if_file_exists().await;
        return Json(serde_json::json!(
            {
                "result":"done",
            }
        ));
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

    async fn check_if_file_exists() -> Result<(), std::io::Error> {
        let dir = fs::read_dir("./static")?;   
        println!("{:?}", dir);
        for entry in dir {
                let entry = entry?;
                let path = entry.path();
        
                print!("{:?} -\t", path.file_name().unwrap_or_default());
            }

        Ok(())
    }
}