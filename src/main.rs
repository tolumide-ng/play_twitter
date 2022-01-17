use twitar::helpers::hmac_signature::{AuthorizeRequest, Signature, ApiCallMethod};

// use twitar::setup::credentials::Credentials;

fn main() {
    let ab = AuthorizeRequest {
        include_entities: String::from("true"),
        oauth_consumer_key: String::from("xvz1evFS4wEEPTGEFPHBog"),
        oauth_nonce: String::from("kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg"),
        oauth_signature_method: String::from("HMAC-SHA1"),
        oauth_timestamp: String::from("1318622958"),
        oauth_token: String::from("370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb"),
        oauth_version: String::from("1.0"),
        base_url: String::from("https://api.twitter.com/1.1/statuses/update.json"),
        method: ApiCallMethod::POST,
    };
    let abc = Signature::new(ab);
    println!("Hello, world! {:#?}", abc);
    
}
