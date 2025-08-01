use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use tokio::sync::Mutex;

use crate::{controller::auth::AuthController, Connections};

#[derive(Clone)]
pub struct AuthRouter {}
impl AuthRouter {
    
    pub fn new(router: Router<Arc<Mutex<Connections>>>, connections: Arc<Mutex<Connections>>) -> Router {        
        return router
                .route("/login", post(AuthController::login_user))
                    .route("/reg", post(AuthController::reg_user))
                        // .route("/verify", get(AuthController::email_verify))
                        .route("/verify/ping", get(AuthController::email_ping_verify))
                        .with_state(connections);
    }   
}
