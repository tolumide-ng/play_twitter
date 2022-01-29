use hyper::{Response, Body};
use serde::{Serialize, Deserialize};

pub type ApiResponse = http::Result<Response<Body>>;

pub struct TApiResponse {
    
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponseBody<T> {
    message: String,
    body: Option<T>
}


impl<T: Serialize> ApiResponseBody<T> {
    pub fn new(message: String, body: Option<T>) -> String {
        let response= Self {
            message,
            body,
        };

        let res= serde_json::to_string(&response).unwrap();
        res
    }
}
