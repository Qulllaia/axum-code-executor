use serde::de::{Deserialize};

#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct CreateCodeRequest {
    pub workspace_name: Option<String>,
    pub code: Option<String>,
    pub file_name: Option<String>,
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
}