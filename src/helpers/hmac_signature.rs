// https://developer.twitter.com/en/docs/authentication/oauth-1-0a/creating-a-signature

use secrecy::{ExposeSecret, Secret};
use sha1::{Sha1};
use urlencoding::encode;
use std::{collections::HashMap};
use hmac::{Hmac, Mac};

#[derive(Default, Debug)]
pub struct Signature {
    parameter_string: Option<String>,
    base_string: Option<Secret<String>>,
    signing_key: Option<Secret<String>>
}


// to be removed later -- for practise only
#[derive(Debug, Clone)]
pub struct AuthorizeRequest {
    /// Should be boolean but for Rust types // todo!() consider implementing this as an enum for boolean with a From trait
    pub include_entities: String,
    /// Identifies the application making the request (twitter app developer portal)
    pub oauth_consumer_key: String,
    /// Unique token that should be generated by this app, Twitter will use this value to determine if a request has been submitted multiple times
    pub oauth_nonce: String, 
    /// the oauth signature method used by Twitter is HMAC-SHA1
    pub oauth_signature_method: String,
    /// Number of seconds since Unix epoch at the point the request is generated
    pub oauth_timestamp: String,
    /// You can initially use the access token of the twitter account that owns the twitter developer app but if you want to make a request on behalf
    /// of a different Twitter account owner, the account owner must grant you access by using the 3-legged OAuth flow (which we are currently 
    /// trying to generate in this case)
    /// More here: https://developer.twitter.com/en/docs/authentication/oauth-1-0a
    pub oauth_token: Option<Secret<String>>,
    pub oauth_version: String,
    pub base_url: String,
    pub method: ApiCallMethod,
    pub consumer_secret: Secret<String>,
    // pub oauth_token_secret: Option<Secret<String>>, // OAuth token secret || token secret
}

// Request Token Secret === oauth_token_secret


#[derive(Debug, Clone)]
pub enum ApiCallMethod {
    POST,
    DELETE,
    GET,
}

impl From<ApiCallMethod> for String {
    fn from(method: ApiCallMethod) -> Self {
        match method {
            ApiCallMethod::DELETE => "DELETE".to_string(),
            ApiCallMethod::GET => "GET".to_string(),
            ApiCallMethod::POST => "POST".to_string(),
        }
    }
}


impl Signature {
    pub fn new(request: AuthorizeRequest) -> Self {
        let mut signature: Signature = Default::default();

        let parameter_string = signature.get_parameter(request.clone());
        signature.parameter_string = Some(parameter_string);
        
        let base_string = signature.get_base_string(request.clone());
        signature.base_string = Some(base_string);

        let signing_key = signature.get_signing_key(request);
        signature.signing_key = Some(signing_key);

        let ad = signature.calculate_signature();

        signature

    }

    fn get_parameter(&self, request: AuthorizeRequest) -> String {
        let mut request_params: HashMap<&str, String> = HashMap::new();
        let AuthorizeRequest {include_entities, oauth_consumer_key, oauth_nonce,
            oauth_signature_method, oauth_timestamp, oauth_token, oauth_version, ..
        } = request;

        request_params.insert("include_entities", include_entities);
        request_params.insert("oauth_consumer_key", oauth_consumer_key);
        request_params.insert("oauth_nonce", oauth_nonce);
        request_params.insert("oauth_signature_method", oauth_signature_method);
        request_params.insert("oauth_timestamp", oauth_timestamp);
        request_params.insert("oauth_token", oauth_token.unwrap().expose_secret().to_string());
        request_params.insert("oauth_version", oauth_version);


        let mut encoded_r_params = vec![];

        for (key, val) in request_params.iter() {
            encoded_r_params.push((encode(key), encode(val)))
        }

        encoded_r_params.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut parameter_string = String::from("");

        encoded_r_params.iter().enumerate().for_each(|(index,val)| {
            let mut key_val = format!("{}={}", val.0, val.1);
            
            if index != encoded_r_params.len() - 1{
                key_val.push_str("&");
            }

            parameter_string.push_str(key_val.as_str());
        });

        parameter_string
    }

    fn get_base_string(&self, request: AuthorizeRequest) -> Secret<String> {
        let mut base_string = String::from("");

        base_string.push_str(format!("{}&", String::from(request.method)).as_str());
        base_string.push_str(format!("{}&", encode(request.base_url.as_str())).as_str());

        let parameter_string = format!("{}", encode(self.parameter_string.as_ref().unwrap().as_str()));
        base_string.push_str(parameter_string.as_str());

        Secret::new(base_string)
    }

    fn get_signing_key(&self, request: AuthorizeRequest) -> Secret<String> {
        let mut signing_key = String::new();

        let consumer_secret = format!("{}&", encode(request.consumer_secret.expose_secret().as_str()));
        signing_key.push_str(consumer_secret.as_str());
        
        if request.oauth_token.is_some() {
            let token_secret = encode(request.oauth_token.unwrap().expose_secret()).to_string();
            signing_key.push_str(token_secret.as_str());
        }

        Secret::new(signing_key)
    }

    fn calculate_signature(&self) -> String{
        type HmacSha1 = Hmac<Sha1>;      

        let parameter_string = self.parameter_string.as_ref().unwrap().as_bytes();
        let base_string = self.base_string.as_ref().unwrap().expose_secret().as_bytes();

        let mut mac = HmacSha1::new_from_slice(parameter_string).expect("Wrong key length");

        mac.update(base_string);

        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        let signing_key =  base64::encode(code_bytes);

        return signing_key
    }

}