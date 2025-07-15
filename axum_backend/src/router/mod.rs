use std::sync::Arc;

use crate::controller::{ExecuteController};

use axum::{routing::{get, patch, post}, Router};
use deadpool_postgres::{Manager, Object};

#[derive(Clone)]
pub struct AppState {
    pub executor: Arc<Object>,
}

pub struct ExecuteRouter {}
impl ExecuteRouter {

    pub fn new(router: Router, database_connection: Object) -> Router {
        let _ = database_connection;
        let executor = Arc::new(database_connection);
        
        let state = AppState { executor };
        return router
            .route("/execute_file/:id", get(ExecuteController::execute_file).with_state(state.clone()))
                .route("/create_file", post(ExecuteController::create_file).with_state(state.clone()))
                        .route("/update_file", patch(ExecuteController::update_file).with_state(state.clone()));
    }   
}
