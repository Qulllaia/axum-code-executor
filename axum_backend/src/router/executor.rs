use std::ops::Deref;
use std::sync::{Arc};
use axum::middleware;
use tokio::sync::Mutex;
use crate::controller::executor::ExecuteController;
use crate::{Connections};

use axum::{routing::{delete, get, patch, post}, Router};
use deadpool_postgres::{Object};
use redis::Connection;
use crate::middleware::auth_middleware;

#[derive(Clone)]
pub struct ExecuteRouter {}
impl ExecuteRouter {

    pub fn new(router: Router<Arc<Mutex<Connections>>>, connections:  Arc<Mutex<Connections>>) -> Router {
        return router
            .route("/get_files/{user_id}", get(ExecuteController::get_files))
                .route("/execute_file", get(ExecuteController::execute_file))
                    .route("/delete_file/{id}", delete(ExecuteController::delete_file))
                        .route("/create_file", post(ExecuteController::create_file))
                            .route("/update_file", patch(ExecuteController::update_file))
                                .layer(middleware::from_fn(auth_middleware))
                                    .with_state(connections);

    }   
}
