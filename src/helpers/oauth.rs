use std::{time::SystemTime, collections::HashMap, borrow::Cow};

use urlencoding::{encode, decode};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use hyper::{Method};
use secrecy::{Secret};
use uuid::Uuid;

use crate::helpers::request::SignedHeader;

use super::{params::{KeyPair, RequestParams}};



#[derive(Debug, Clone)]
pub enum OAuthAddOn {
    /// oauth_callback: callback url for generating request token.
    Callback(String),
    /// oauth_verifier: verifier for generating access token.
    Verifier(String),
    None
}

impl Default for OAuthAddOn {
    fn default() -> Self {
        OAuthAddOn::None
    }
}


impl OAuthAddOn {
    /// Returns oauth_callback parameter
    pub fn with_callback(&self) -> Option<String> {
        match self {
            Self::Callback(url) => Some(url.to_string()),
            _ => None,
        }
    }

    pub fn with_verifier(&self) -> Option<String> {
        match self {
            Self::Verifier(v) => Some(v.to_string()),
            _ => None
        }
    }
}



// to be removed later -- for practise only
#[derive(Debug, Clone)]
pub struct OAuthParams {
    pub consumer: KeyPair,
    pub nonce: String,
    pub timestamp: u64,
    pub token: Option<KeyPair>,
    pub addon: OAuthAddOn,
}


impl OAuthParams {
    fn new() -> Self {
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(e) => panic!("SystemTime before UNIX EPOCH!"),
        };

        Self {
            consumer: KeyPair::empty(),
            nonce: Uuid::new_v4().to_string(),
            timestamp,
            token: None,
            addon: OAuthAddOn::None,
        }
    }

    pub fn from_keys(consumer: KeyPair, token: Option<KeyPair>) -> Self {
        Self {
            consumer, 
            token,
            ..Self::new()
        }
    }

    /// Adds callback_url or verifier based on argument
    pub fn with_addon(self, addon: OAuthAddOn) -> Self {
        Self {
            addon,
            ..self
        }
    }

    /// 1.0 Authentication on behalf of users
    pub fn sign_request(self, method: &Method, params: Option<&RequestParams>, url: &str, req_query: String) -> SignedHeader {
        let the_params = params
            .cloned()
            .unwrap_or_default()
            .add_opt_param("oauth_callback", self.addon.with_callback().map(|k| k))
            .add_param("oauth_consumer_key", self.consumer.key.clone())
            .add_param("oauth_nonce", self.nonce.clone())
            .add_param("oauth_signature_method", "HMAC-SHA1")
            .add_param("oauth_timestamp", self.timestamp.to_string())
            .add_opt_param("oauth_token", self.token.clone().map(|k| k.key))
            .add_opt_param("oauth_verifier", self.addon.with_verifier().map(|k| k))
            .add_param("oauth_version", "1.0");

        let mut query: Vec<String> = the_params
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect::<Vec<_>>();

        query.sort();

        let params_str = query.join("&");
        
        // get the signature base string
        let uri = format!("{}{}", url, decode(&req_query).unwrap());
        let base_str = format!("{}&{}&{}",
            method.as_str(),
            encode(uri.as_str()),
            // encode(url),
            encode(&params_str)
        );

        // generate signing key
        let key = format!("{}&{}", 
            encode(&self.consumer.value), 
            encode(&self.token.as_ref().unwrap_or(&KeyPair::new("", "")).value)
        );

        // calculate the signature
        type HmacSha1 = Hmac::<Sha1>;

        let mut mac = HmacSha1::new_from_slice(key.as_bytes()).expect("Wrong key length");
        mac.update(base_str.as_bytes());

        let signed_key = base64::encode(mac.finalize().into_bytes());

        let mut all_params = vec![
            ("oauth_consumer_key", self.consumer.key.clone()), // MUST BE THE FIRST ITERM IN THE VEC (USED ON THE GET_PARAMS IMPLEMENTATION ON PARAMSLIST)
            ("oauth_nonce", self.nonce.clone().into()),
            ("oauth_signature", signed_key.into()),
            ("oauth_signature_method", "HMAC-SHA1".into()),
            ("oauth_timestamp", self.timestamp.to_string().into()),
            ("oauth_version", "1.0".into()),
        ];

        match &self.addon {
            OAuthAddOn::Callback(c) => {
                all_params.push(("oauth_callback", c.clone().into()));
            },
            OAuthAddOn::Verifier(v) => {
                all_params.push(("oauth_verifider", v.clone().into()));
            },
            OAuthAddOn::None => {}
        };

        if let Some(token) = &self.token {
            all_params.push(("token", token.key.clone()));
        }

        SignedHeader {
            params: all_params
        }
    }


    /// 2.0 Authentication on behalf of users (confidential clients)
 
}
