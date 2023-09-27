use std::{collections::HashMap, fs, io, path::Path, time::SystemTime};

use chrono::{DateTime, Duration, FixedOffset, Utc};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct TempToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub created_at: String,
}

impl TempToken {
    const TEMP_CREDENTIAL_PATH: &str = ".temp/token.json";

    fn load_to_file() -> Result<TempToken, io::Error> {
        let token_data = fs::read_to_string(Self::TEMP_CREDENTIAL_PATH)?;
        let token: TempToken = serde_json::from_str(&token_data)?;

        Ok(token)
    }

    fn save_to_file(&self) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(self)?;

        let path = Path::new(Self::TEMP_CREDENTIAL_PATH);
        if let Some(dir) = path.parent() {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
        }

        fs::write(path, json)?;
        Ok(())
    }

    fn valid_token(&self) -> bool {
        let datetime_with_offset =
            DateTime::<FixedOffset>::parse_from_rfc3339(&self.created_at).unwrap();
        let datetime_utc: DateTime<Utc> = datetime_with_offset.with_timezone(&Utc);
        let now = Utc::now();

        now < datetime_utc + Duration::seconds(self.expires_in as i64)
    }
}

const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

pub async fn get_access_token() -> Result<TokenResponse, reqwest::Error> {
    if let Ok(token) = TempToken::load_to_file() {
        if token.valid_token() {
            Ok(TokenResponse {
                access_token: token.access_token,
                token_type: token.token_type,
                expires_in: token.expires_in,
                refresh_token: token.refresh_token,
                scope: token.scope,
            })
        } else {
            todo!("refresh token");
        }
    } else {
        let data = fs::read_to_string("credentials.json").unwrap();
        let credentials: Credentials = serde_json::from_str(&data).unwrap();

        let client_id = &credentials.installed.client_id;
        let client_secret = &credentials.installed.client_secret;
        let (pkce_verifier, auth_code) = get_auth_code(client_id, client_secret);

        let token =
            get_access_token_internal(client_id, client_secret, &auth_code, pkce_verifier.secret())
                .await?;
        save_to_tmpfile(&token);

        Ok(token)
    }
}

fn save_to_tmpfile(token: &TokenResponse) {
    let date_time: DateTime<Utc> = SystemTime::now().into();
    let credentials = TempToken {
        access_token: token.access_token.clone(),
        token_type: token.token_type.clone(),
        expires_in: token.expires_in,
        refresh_token: token.refresh_token.clone(),
        scope: token.scope.clone(),
        created_at: date_time.to_rfc3339(),
    };
    let _ = credentials.save_to_file();
}

fn get_auth_code(client_id: &str, client_secret: &str) -> (oauth2::PkceCodeVerifier, String) {
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
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string()).unwrap());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar.readonly".to_string(),
        )) // スコープを設定
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("次のURLをブラウザで開いて認証してください:\n{}", auth_url);

    let mut auth_code = String::new();

    println!("認証コードをペーストしてEnterを入力:");
    std::io::stdin().read_line(&mut auth_code).unwrap();
    let auth_code = auth_code.trim();

    (pkce_verifier, auth_code.to_string())
}

async fn get_access_token_internal(
    client_id: &str,
    client_secret: &str,
    auth_code: &str,
    code_verifier: &str,
) -> Result<TokenResponse, reqwest::Error> {
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("code", auth_code);
    params.insert("redirect_uri", REDIRECT_URI);
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
