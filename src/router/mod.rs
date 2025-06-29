use crate::controller::{ExecuteController};

use axum::{routing::{get, put}, Router};

pub struct ExecuteRouter {}
impl ExecuteRouter {

    pub fn new(router: Router) -> Router {
        return router
            .route("/create_file/", put(ExecuteController::create_file))
                .route("/execute_file", get(ExecuteController::execute_file))
    }   
}
