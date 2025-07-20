use std::sync::Arc;

use crate::controller::{ExecuteController};

use axum::{routing::{delete, get, patch, post}, Router};
use deadpool_postgres::{Object};

#[derive(Clone)]
pub struct ExecuteRouter {}
impl ExecuteRouter {

    pub fn new(router: Router<Arc<Object>>, database_connection: Object) -> Router {
        let _ = database_connection;
        let executor = Arc::new(database_connection);
        
        return router
            .route("/get_files/{user_id}", get(ExecuteController::get_files))
                .route("/execute_file/{id}", get(ExecuteController::execute_file))
                    .route("/delete_file/{id}", delete(ExecuteController::delete_file))
                        .route("/create_file", post(ExecuteController::create_file))
                                .route("/update_file", patch(ExecuteController::update_file))
                                    .with_state(executor);
    }   
}
