use serde::de::{Deserialize};

#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct CreateCodeRequest {
    pub workspace_name: Option<String>,
    pub input: Option<String>,
    pub code: Option<String>,
    pub file_name: Option<String>,
    pub user_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ExecuteParams {
    pub id: String,
    pub args: Option<String>, 
}

#[derive(Debug)]
#[derive(serde::Serialize)]
pub struct GetFilesResponse {
    pub workspace_uid: String, 
    pub workspace_name: String, 
    pub user_id: i64, 
    pub code: String,
}
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CacheData {
    pub code: String, 
    pub code_output: String, 
    pub code_error: String, 
    pub input: String,
}

#[derive(serde::Deserialize)]
pub struct UserData {
    pub email: String, 
    pub password: String,
}
#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: i64,  
    pub exp: usize,   
}

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthBody {
    pub access_token: String,
}