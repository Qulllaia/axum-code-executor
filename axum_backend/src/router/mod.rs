use std::sync::{Arc};
use tokio::sync::Mutex;
use crate::{controller::ExecuteController, Connections};

use axum::{routing::{delete, get, patch, post}, Router};
use deadpool_postgres::{Object};
use redis::Connection;

#[derive(Clone)]
pub struct ExecuteRouter {}
impl ExecuteRouter {

    pub fn new(router: Router<Arc<Mutex<Connections>>>, connections: Connections) -> Router {
        let connections = Arc::new(Mutex::new(connections));
        
        return router
            .route("/get_files/{user_id}", get(ExecuteController::get_files))
                .route("/execute_file/{id}", get(ExecuteController::execute_file))
                    .route("/delete_file/{id}", delete(ExecuteController::delete_file))
                        .route("/create_file", post(ExecuteController::create_file))
                            .route("/update_file", patch(ExecuteController::update_file))
                                .with_state(connections);

    }   
}
