use crate::controller::{ExecuteController};

use axum::{routing::{get, patch, post}, Router};

pub struct ExecuteRouter {}
impl ExecuteRouter {

    pub fn new(router: Router) -> Router {
        return router
        .route("/execute_file/:id", get(ExecuteController::execute_file))
            .route("/create_file", post(ExecuteController::create_file))
                    .route("/update_file", patch(ExecuteController::update_file))
    }   
}
