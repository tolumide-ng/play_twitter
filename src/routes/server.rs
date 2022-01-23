use hyper::{Request, Body, Response, Method, StatusCode};

use crate::{helpers::response::ApiResponse};
use crate::api::{not_found, authorize_bot, health_check, handle_redirect};

#[derive(Debug, serde::Deserialize)]
struct ABody {
    ab: String,
    cd: String,
}

impl From<hyper::Body> for ABody {
    fn from(_: hyper::Body) -> Self {
        Self {
            ab: "".to_string(),
            cd: "".to_string()
        }
    }
}


pub async fn routes(req: Request<Body>) -> ApiResponse {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => health_check(),
        (&Method::GET, "/enable") => authorize_bot().await,
        (&Method::GET, "/twitter/oauth") => handle_redirect().await,
        _ => {
            not_found()
        }
    }
}