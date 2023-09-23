use std::{collections::HashMap, fs};

use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Credentials {
    installed: Installed,
}

#[derive(Debug, Serialize, Deserialize)]
struct Installed {
    client_id: String,
    client_secret: String,
}

const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

async fn get_access_token_internal(
    client_id: &str,
    client_secret: &str,
    auth_code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenResponse, reqwest::Error> {
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("code", auth_code);
    params.insert("redirect_uri", redirect_uri);
    params.insert("grant_type", "authorization_code");
    params.insert("code_verifier", code_verifier);

    let response: TokenResponse = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

pub async fn get_access_token() -> Result<TokenResponse, reqwest::Error> {
    let data = fs::read_to_string("credentials.json").unwrap();
    let credentials: Credentials = serde_json::from_str(&data).unwrap();

    let client_id = &credentials.installed.client_id;
    let client_secret = &credentials.installed.client_secret;
    let redirect_uri = "urn:ietf:wg:oauth:2.0:oob";
    let google_client_id = ClientId::new(client_id.to_string());
    let google_client_secret = ClientSecret::new(client_secret.to_string());

    let auth_url =
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap();

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string()).unwrap());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar.readonly".to_string(),
        )) // スコープを設定
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!(
        "Please open the following URL in your browser:\n{}",
        auth_url
    );

    // コンソールからユーザーが入力した認証コードを取得
    let mut auth_code = String::new();
    println!("Please enter the authorization code you received:");
    std::io::stdin().read_line(&mut auth_code).unwrap();
    let auth_code = auth_code.trim();

    print!("auth_code: {}", auth_code);

    return get_access_token_internal(
        client_id,
        client_secret,
        auth_code,
        redirect_uri,
        pkce_verifier.secret(),
    )
    .await;
}
