use std::{fs, path::Path, time::SystemTime};

use anyhow::Result;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, RedirectUrl, RefreshToken, Scope, StandardTokenResponse, TokenResponse,
    TokenUrl,
};

use serde::{Deserialize, Serialize};

const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
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

    fn load_to_file() -> Result<TempToken> {
        let token_data = fs::read_to_string(Self::TEMP_CREDENTIAL_PATH)?;
        let token: TempToken = serde_json::from_str(&token_data)?;

        Ok(token)
    }

    fn save_to_file(&self) -> Result<()> {
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
        let datetime_with_offset = DateTime::<FixedOffset>::parse_from_rfc3339(&self.created_at);
        let datetime_utc: DateTime<Utc> = if let Ok(dt) = datetime_with_offset {
            dt.with_timezone(&Utc)
        } else {
            return false;
        };
        let now = Utc::now();
        now < datetime_utc + Duration::seconds(self.expires_in as i64)
    }
}

struct AuthClient {
    client: oauth2::basic::BasicClient,
}

impl AuthClient {
    fn new(client_id: &str, client_secret: &str) -> Self {
        let google_client_id = ClientId::new(client_id.to_string());
        let google_client_secret = ClientSecret::new(client_secret.to_string());

        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid auth url");
        let token_url = TokenUrl::new(TOKEN_ENDPOINT.to_string()).expect("Invalid token url");

        let client = BasicClient::new(
            google_client_id,
            Some(google_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new(REDIRECT_URI.to_string()).expect("Invalid redirect URL"),
        );
        Self { client }
    }

    fn get_auth_code(&self, pkce_challenge: PkceCodeChallenge) -> Result<String> {
        let (auth_url, _csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/calendar.readonly".to_string(),
            )) // スコープを設定
            .set_pkce_challenge(pkce_challenge)
            .url();

        println!("次のURLをブラウザで開いて認証してください:\n{}", auth_url);

        let mut auth_code = String::new();

        println!("認証コードをペーストしてEnterを入力:");
        std::io::stdin().read_line(&mut auth_code)?;

        Ok(auth_code.trim().to_string())
    }

    async fn get_access_token(
        &self,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let auth_code = self.get_auth_code(pkce_challenge)?;
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(auth_code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;
        Ok(token_response)
    }

    async fn get_token_from_refresh(
        &self,
        refresh_token: &str,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let refresh_token = RefreshToken::new(refresh_token.to_string());
        let token_response = self
            .client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?;
        Ok(token_response)
    }
}

pub async fn get_access_token() -> Result<Token> {
    // TOOD: creadentials.jsonが無いときのハンドリング
    let data = fs::read_to_string("credentials.json")?;
    let credentials: Credentials = serde_json::from_str(&data)?;
    let client_id = &credentials.installed.client_id;
    let client_secret = &credentials.installed.client_secret;

    if let Ok(token) = TempToken::load_to_file() {
        if token.valid_token() {
            Ok(Token {
                access_token: token.access_token,
                token_type: token.token_type,
                expires_in: token.expires_in,
                refresh_token: token.refresh_token,
                scope: token.scope,
            })
        } else {
            let client = AuthClient::new(client_id, client_secret);
            let refresh_token_str = token.refresh_token.unwrap();
            let token_response = client.get_token_from_refresh(&refresh_token_str).await?;
            let token = save_to_tmpfile(&token_response)?;
            Ok(token)
        }
    } else {
        let client = AuthClient::new(client_id, client_secret);
        let token = client.get_access_token().await?;
        let token = save_to_tmpfile(&token)?;

        Ok(token)
    }
}

fn save_to_tmpfile(
    token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
) -> Result<Token> {
    let date_time: DateTime<Utc> = SystemTime::now().into();
    let temp_token = TempToken {
        access_token: token.access_token().secret().clone(),
        token_type: token.token_type().as_ref().to_string(),
        expires_in: token.expires_in().unwrap().as_secs(),
        refresh_token: Some(token.refresh_token().unwrap().secret().to_string()),
        scope: Some(
            token
                .scopes()
                .unwrap()
                .first()
                .unwrap()
                .as_ref()
                .to_string(),
        ),
        created_at: date_time.to_rfc3339(),
    };

    let _ = temp_token.save_to_file();

    Ok(Token {
        access_token: temp_token.access_token,
        token_type: temp_token.token_type,
        expires_in: temp_token.expires_in,
        refresh_token: Some(temp_token.refresh_token.unwrap()),
        scope: Some(
            temp_token
                .scope
                .unwrap()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(","),
        ),
    })
}
