use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use std::{collections::HashMap, fs, sync::Arc};

use tokio::sync::Mutex;

use std::path::Path as p;
use std::fs::File as f;
use uuid::Uuid;
use std::io::{Write};
use tokio::process::Command;
use crate::{auth_utils::AuthUtils, Connections};
use crate::cache::Cache;
use crate::types::{ GetFilesResponse, CreateCodeRequest, CacheData, ExecuteParams};

static DIR_PATH: &'static str = "static";  
#[derive(Debug)]
pub struct ExecuteController;
impl ExecuteController {


    pub async fn delete_file(
        State(state): State<Arc<Mutex<Connections>>>,
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

        match (*state).lock().await.database.execute("DELETE FROM \"Workspace\" WHERE workspace_uid = $1", &[&id]).await {
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

        // Почему-то exe файлы создаются в формате " {название}"
        // Пока не приоритет, потом починю 
        if p::new(&format!("./static/ {uid}.exe")).exists() {
            fs::remove_file(format!("./static/ {uid}.exe"))?;
        }

        Ok(())
    }

    pub async fn execute_file(
        State(connections): State<Arc<Mutex<Connections>>>,
        Query(params): Query<ExecuteParams>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        // println!("{:?}", params.args);
        // println!("{:?}", params.id);
         
        let id = params.id;
        let mut args= String::new();

        if params.args.is_some() { 
            args = params.args.unwrap().replace("\\n", " ");
        }

        let mut connections = (*connections).lock().await;
        let result = Cache::check_filed_existance(&mut connections, &id).await;
        let mut database_code: String = String::new();
        let query = connections.database.query("SELECT code FROM \"Workspace\" WHERE workspace_uid = $1", &[&id]).await;
        match query {
            Ok(rows) => {
                let rows_data: Vec<String> = rows.iter().map(|row| {
                    row.get("code")
                }).collect();

                    database_code = rows_data[0].clone();
                    // println!("==== {:?}", database_code);
                },
            Err(error) => return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!(
                {
                    "result":"error",
                    "error": error.to_string()
                }
            ))),
        }

        if result {
            
            let cache_code_data = Cache::get_data_by_field(&mut connections, &id).await; 
            let cache_code = serde_json::from_str::<CacheData>(&cache_code_data).unwrap();

            if database_code == cache_code.code && args == cache_code.input {
                return (StatusCode::OK, Json(serde_json::json!(
                    {
                        "result":"same",
                        "code_output": cache_code.code_output, 
                        "code_error": cache_code.code_error
                    }
                )));
            }
        }

        let work_dir = p::new("static");

        let check_gcc = Command::new("gcc")
            .current_dir(&work_dir)
            .arg(format!("{}.c", &id))
            .arg(format!("-o{}.exe", &id))
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
            .arg(format!("echo {} | {}.exe", &args, &id))  
            .output()
            .await;
        
        match exec_output {
            Ok(result) => {
                // println!("{:?}", database_code.len());
                if database_code.len() > 0 {
                    let cache_code_data = CacheData {
                        code: database_code.to_owned(),
                        code_error: String::from_utf8_lossy(&result.stderr).to_string(),
                        code_output: String::from_utf8_lossy(&result.stdout).to_string(),
                        input: args
                    };
                    
                    let _:() = Cache::set_data_by_field(&mut connections, &id, &serde_json::to_string(&cache_code_data).unwrap()).await;
                }
                
                return (StatusCode::OK, Json(serde_json::json!(
                    {
                        "result": "done",
                        "code_output": String::from_utf8_lossy(&result.stdout),
                        "code_error": String::from_utf8_lossy(&result.stderr),
                    }
                )))
            },
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!(
                {
                    "result":"error",
                    "error": e.to_string()
                }
            ))),
        }


    }

    pub async fn create_file(
        jar: CookieJar,
        State(state): State<Arc<Mutex<Connections>>>,
        Json(file_data): Json<CreateCodeRequest>
    ) -> (StatusCode, Json<serde_json::Value>) {

        let all_cookies = jar.iter().map(|c| (c.name(), c.value())).collect::<HashMap<&str, &str>>();

        // println!("Все куки: {:?}", all_cookies);

        let user_id = AuthUtils::validate_token(all_cookies["jwt_token"]).unwrap().sub;

        let code = file_data.code.unwrap();
        let workspace_name = file_data.workspace_name.unwrap();

        let file_id: String = Uuid::new_v4().to_string() .replace('-', "");
        
        match (*state).lock().await.database.execute("INSERT INTO \"Workspace\" (workspace_uid, workspace_name, user_id, code) VALUES ($1, $2, $3, $4)", 
                &[&file_id, &workspace_name, &user_id, &code]).await {
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
        State(state): State<Arc<Mutex<Connections>>>,
        Json(file_data): Json<CreateCodeRequest>
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut connections = (*state).lock().await;

        let file_id: &String = &file_data.file_name.unwrap().replace('-', "");
        let code = &file_data.code.unwrap();

        let result = Cache::check_filed_existance(&mut connections, file_id).await;
              
        if result {
            let redis_value: String  = Cache::get_data_by_field(&mut connections, file_id).await;
            if code == &redis_value {
                return (StatusCode::OK, Json(serde_json::json!(
                    {
                        "result":"done",
                    }))
                )
            }
        }
        
        let searching_file_result = Self::check_if_file_exists(file_id).await;
        match connections.database.execute("update \"Workspace\" set code = $2 where workspace_uid = $1", &[&file_id, code]).await {
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

    // #[axum::debug_handler]
    pub async fn get_files(
        jar: CookieJar,
        State(state): State<Arc<Mutex<Connections>>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        
        let all_cookies = jar.iter().map(|c| (c.name(), c.value())).collect::<HashMap<&str, &str>>();

        let user_id = AuthUtils::validate_token(all_cookies["jwt_token"]).unwrap().sub;

        let conn = state.lock().await;
        match conn.database.query("SELECT * FROM \"Workspace\" WHERE user_id = $1", &[&user_id]).await {
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