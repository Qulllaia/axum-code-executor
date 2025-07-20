mod types;
use axum::{extract::{FromRequest, Path, Request, State}, response::IntoResponse, Error, Json, http::StatusCode};
use deadpool_postgres::{Manager, Object};
use serde_json::json;
use std::{fmt, fs, sync::Arc};
use types::CreateCodeRequest;
use std::path::Path as p;
use std::fs::File as f;
use uuid::Uuid;
use std::env;
use std::io::{Write};
use tokio::process::Command;

use crate::controller::types::GetFilesResponse;

static DIR_PATH: &'static str = "static";  
#[derive(Debug)]
pub struct ExecuteController;
impl ExecuteController {


    pub async fn delete_file(
        State(state): State<Arc<Object>>,
        Path(id): Path<String>
    ) -> (StatusCode, Json<serde_json::Value>) {

        match Self::file_delete(&id).await {
            Ok(_) => {},
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                    {
                        "result":"error",
                        "error":format!("{:?}", error),
                    }
                )));
            }
        }

        match (*state).execute("DELETE FROM \"Workspace\" WHERE workspace_uid = $1", &[&id]).await {
            Ok(_) => {},
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                    {
                        "result":"error",
                        "code_error": format!("{:?}", &error),
                    }
                )));
            }
        }
        return (StatusCode::OK, Json(serde_json::json!(
            {
                "result":"done",
                "code_error": "done"
            }
        ))); 
    }

    async fn file_delete(uid: &String) -> Result<(), std::io::Error> {

        if p::new(&format!("./static/{uid}.c")).exists() {
            fs::remove_file(format!("./static/{uid}.c"))?;
        }

        if p::new(&format!("./static/{uid}.exe")).exists() {
            fs::remove_file(format!("./static/{uid}.exe"))?;
        }

        Ok(())
    }

    pub async fn execute_file(
        Path(id): Path<String>
    ) -> (StatusCode, Json<serde_json::Value>) {

        let work_dir = p::new("static");

        let check_gcc = Command::new("gcc")
            .current_dir(&work_dir)
            .arg(format!("{}.c", &id))
            .arg(format!("-o {}.exe", &id))
            .output()
            .await;

         match check_gcc {
            Ok(compile_result) => {
                if compile_result.stderr.len() > 0 {
                    return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!(
                        {
                            "result":"error",
                            "code_error": String::from_utf8_lossy(&compile_result.stderr)
                        }
                    )));    
                }
            },
            Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            ))),
        }

        let exec_output = Command::new("cmd")
            .current_dir(&work_dir)
            .arg("/C")
            .arg(format!(" {}.exe", &id))
            .output()
            .await;
        
        match exec_output {
            Ok(result) => return (StatusCode::OK, Json(serde_json::json!(
                {
                    "result": "done",
                    "code_output": String::from_utf8_lossy(&result.stdout),
                    "code_error": String::from_utf8_lossy(&result.stderr),
                }
            ))),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            ))),
        }
    }

    pub async fn create_file(
        State(state): State<Arc<Object>>,
        Json(file_data): Json<CreateCodeRequest>
    ) -> (StatusCode, Json<serde_json::Value>) {

        let code = file_data.code.unwrap();
        let workspace_name = file_data.workspace_name.unwrap();

        let file_id: String = Uuid::new_v4().to_string() .replace('-', "");
        
        match (*state).execute("INSERT INTO \"Workspace\" (workspace_uid, workspace_name, user_id, code) VALUES ($1, $2, $3, $4)", 
                &[&file_id, &workspace_name, &1_i64, &code]).await {
            Ok(_) => {},
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                    {
                        "result":"error",
                        "error":format!("{:?}", error),
                    }
                )));
            }
        };

        match Self::file_generator(&code, &file_id).await {
            Ok(_) => {},
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                    {
                        "result":"error",
                        "error":format!("{:?}", error),
                    }
                )));
            }
        }

        return (StatusCode::OK, Json(serde_json::json!(
            {
                "result":"done",
                "file_name":file_id,
            }
        )));
    }

    pub async fn update_file(
        State(state): State<Arc<Object>>,
        Json(file_data): Json<CreateCodeRequest>
    ) -> (StatusCode, Json<serde_json::Value>) {
        let file_id: &String = &file_data.file_name.unwrap().replace('-', "");
        let code = &file_data.code.unwrap();
        let searching_file_result = Self::check_if_file_exists(file_id).await;
        match (*state).execute("update \"Workspace\" set code = $2 where workspace_uid = $1", &[&file_id, code]).await {
            Ok(_) => {},
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                    {
                        "result":"error",
                        "error":format!("{:?}", error),
                    }
                )));
            }
        };
        match searching_file_result {
            Ok(_) => {
                let _ = Self::file_generator(code, &file_id.to_string()).await;
                return (StatusCode::OK, Json(serde_json::json!(
                {
                    "result":"done",
                }
            )))
            },
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            ))),
        }
    }

    async fn file_generator(code: &String, uid: &String) -> Result<(), std::io::Error> {
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

    pub async fn get_files(
        State(state): State<Arc<Object>>,
        Path(user_id): Path<String>
    ) -> (StatusCode, Json<serde_json::Value>) {

        match (*state).query("SELECT * FROM \"Workspace\" WHERE user_id = $1", &[&user_id.parse::<i64>().unwrap()]).await {
            Ok(rows) => {
                let response: Vec<GetFilesResponse> = rows.iter().map(|row| {
                    GetFilesResponse {
                        workspace_uid: row.get("workspace_uid"),
                        workspace_name: row.get("workspace_name"),
                        user_id: row.get("user_id"),
                        code: row.get("code"),
                    }
                }).collect();

                return (StatusCode::OK, Json(serde_json::json!(
                    {
                        "row": response,
                    }
                )));
            },
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                    {
                        "result":"error",
                        "error":format!("{:?}", error),
                    }
                )));
            }
        }

    }
}