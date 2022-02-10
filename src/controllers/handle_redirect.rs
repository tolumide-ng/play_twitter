use http::Method;
use hyper::{StatusCode, Request};
use redis::{Client as RedisClient};
use serde::{Serialize, Deserialize};
use crate::{helpers::{
    response::{TResult, ApiBody, make_request, ResponseBuilder}, 
    request::{HyperClient}, keyval::KeyVal}, 
    setup::variables::SettingsVars, errors::response::{TError}, middlewares::request_builder::RequestBuilder, interceptor::handle_request::TwitterInterceptor
};


#[derive(Debug, Clone)]
pub struct AccessToken {
    pub state: String,
    pub  code: String,
}

impl AccessToken {
    pub fn validate_state(self, local_state: String) -> TResult<Self> {
        if self.state != local_state {
            return Err(TError::InvalidCredentialError("The state value obtained from the redirect uri does not match the local one".into()));
        }

        Ok(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct AppAccess {
    token_type: String,
    expires_in: i32,
    access_token: String,
    scope: String,
    refresh_token: String,
}

async fn access_token(hyper_client: HyperClient, redis_client: RedisClient, auth_code: String) -> Result<(), TError> {
    let SettingsVars{client_id, oauth2_callback, client_secret, ..} = SettingsVars::new();
    let mut con = redis_client.get_async_connection().await.unwrap();


    let req_body = KeyVal::new().add_list_keyval(vec![
        ("code".into(), auth_code.clone()),
        ("grant_type".to_string(), "authorization_code".into()),
        ("client_id".to_string(), client_id.clone()),
        ("redirect_uri".to_string(), oauth2_callback),
        ("code_verifier".to_string(), redis::cmd("GET").arg(&["tolumide_test_pkce"]).query_async(&mut con).await?)
    ]).to_urlencode();

    let content_type = "application/x-www-form-urlencoded";

    let request = RequestBuilder::new(Method::POST, "https://api.twitter.com/2/oauth2/token".into())
        .with_basic_auth(client_id, client_secret)
        .with_body(req_body, content_type).build_request();

    let (_header, body) = make_request(request, hyper_client.clone()).await?;


    let body: AppAccess = serde_json::from_slice(&body).unwrap();

    redis::cmd("SET").arg(&["tolumide_test_access", &body.access_token]).query_async(&mut con).await?;
    redis::cmd("SET").arg(&["tolumide_refresh_token", &body.refresh_token]).query_async(&mut con).await?;
    redis::cmd("SET").arg(&["tolumide_token_type", &body.token_type]).query_async(&mut con).await?;

    Ok(())
}



pub async fn handle_redirect(req: Request<hyper::Body>, hyper_client: HyperClient, redis_client: RedisClient) -> TResult<ApiBody> {
    let mut con = redis_client.get_async_connection().await?;
    let SettingsVars{state, api_key, ..} = SettingsVars::new();

    println!("EVERYTHING ABOUT THE REQUEST TO THIS ENDPOINT {:#?}", req);

    let query_params = KeyVal::query_params_to_keyval(req.uri())?;

    let is_v1_callback = query_params.verify_present(vec!["oauth_token".into(), "oauth_verifier".into()]);

    match is_v1_callback {
         Ok(k) => {
            let oauth_token: String = redis::cmd("GET").arg(&["oauth_token"]).query_async(&mut con).await?;
            if k.validate("oauth_token".into(),oauth_token.clone()) {
                let verifier = k.get("oauth_verifier").unwrap();
                redis::cmd("SET").arg(&["oauth_verifier", verifier]).query_async(&mut con).await?;

                let header = KeyVal::new().add_list_keyval(vec![
                    ("oauth_consumer_key".into(), api_key),
                    ("oauth_token".into(), oauth_token),
                    ("oauth_verifier".into(), verifier.to_string()),
                ]);

                let req = RequestBuilder::new(Method::POST, "https://api.twitter.com/oauth/access_token".into())
                    .with_header(header).build_request();

                let res = TwitterInterceptor::intercept(make_request(req, hyper_client.clone()).await);

                println!("::::::: THE RESPONSE OBRAINED ::::::: {:#?}", res);

                return ResponseBuilder::new("Access Granted".into(), Some(""), StatusCode::OK.as_u16()).reply();
            }

        }
        Err(e) => {
            // maybe it is a v2 callback
            let is_v2_callback = query_params.verify_present(vec!["code".into(), "state".into()]);

            if let Ok(dict) = is_v2_callback {
                if query_params.validate("state".into(), state) {
                    let code = dict.get("code").unwrap().to_string();
                    access_token(hyper_client.clone(), redis_client, code).await?;

                    return ResponseBuilder::new("Access Granted".into(), Some(""), StatusCode::OK.as_u16()).reply();
                }
            }
        }
    }
    
    ResponseBuilder::new("Bad request".into(), Some(""), StatusCode::BAD_REQUEST.as_u16()).reply()


}
