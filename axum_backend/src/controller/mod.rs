mod types;
use axum::{extract::{FromRequest, Path, Request, State}, response::IntoResponse, Error, Json};
use deadpool_postgres::Object;
use serde_json::json;
use std::fs;
use types::CreateCodeRequest;
use std::path::Path as p;
use std::fs::File as f;
use uuid::Uuid;
use std::env;
use std::io::{Write};
use tokio::process::Command;

use crate::router::AppState;

static DIR_PATH: &'static str = "static";  
#[derive(Debug)]
pub struct ExecuteController;
impl ExecuteController {

    pub fn new(database_connection: Object) -> Self {
        return Self
    }

    pub async fn execute_file(
        State(state): State<AppState>,
        Path(id): Path<String>) -> Json<serde_json::Value> {

        let work_dir = p::new("static");

        let check_gcc = Command::new("gcc")
            .current_dir(&work_dir)
            .arg(format!("{}.c", &id))
            .arg(format!("-o {}.exe", &id))
            .output()
            .await;
        println!("{:?}", check_gcc);
         match check_gcc {
            Ok(compile_result) => {
                if compile_result.stderr.len() > 0 {
                    return  Json(serde_json::json!(
                        {
                            "result":"error",
                            "code_error": String::from_utf8_lossy(&compile_result.stderr)
                        }
                    ))    
                }
            },
            Err(e) => return  Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            )),
        }

        let _ = state.executor.execute("INSERT INTO \"Workspace\" (code) VALUES ($1)", &[&"code sample"]).await;

        let exec_output = Command::new("cmd")
            .current_dir(&work_dir)
            .arg("/C")
            .arg(format!(" {}.exe", &id))
            .output()
            .await;
        
        match exec_output {
            Ok(result) => return Json(serde_json::json!(
                {
                    "result": "done",
                    "code_output": String::from_utf8_lossy(&result.stdout),
                    "code_error": String::from_utf8_lossy(&result.stderr),
                }
            )),
            Err(e) => return  Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            )),
        }
    }

    pub async fn create_file(
        Json(file_data): Json<CreateCodeRequest>
    ) -> Json<serde_json::Value> {

        let file_id: String = Uuid::new_v4().to_string() .replace('-', "");
        let _ = Self::file_generator(file_data.code.unwrap(), &file_id).await;
        return Json(serde_json::json!(
            {
                "result":"done",
                "file_name":file_id,
            }
        ));
    }

    pub async fn update_file(Json(file_data): Json<CreateCodeRequest>) -> Json<serde_json::Value> {
        let file_id: &String = &file_data.file_name.unwrap().replace('-', "");
        let searching_file_result = Self::check_if_file_exists(file_id).await;
        match searching_file_result {
            Ok(_) => {
                let _ = Self::file_generator(file_data.code.unwrap(), &file_id.to_string()).await;
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

    async fn file_generator(code: String, uid: &String) -> Result<(), std::io::Error> {
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