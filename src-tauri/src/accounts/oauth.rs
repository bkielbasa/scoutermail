use base64::Engine;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
    basic::BasicClient,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

pub fn google_config(client_id: &str, client_secret: &str) -> OAuthConfig {
    OAuthConfig {
        provider: "google".into(),
        client_id: client_id.into(),
        client_secret: client_secret.into(),
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
        token_url: "https://oauth2.googleapis.com/token".into(),
        scopes: vec!["https://mail.google.com/".into()],
    }
}

pub fn microsoft_config(client_id: &str, client_secret: &str) -> OAuthConfig {
    OAuthConfig {
        provider: "microsoft".into(),
        client_id: client_id.into(),
        client_secret: client_secret.into(),
        auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".into(),
        token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".into(),
        scopes: vec![
            "https://outlook.office365.com/IMAP.AccessAsUser.All".into(),
            "https://outlook.office365.com/SMTP.Send".into(),
            "offline_access".into(),
        ],
    }
}

/// Start OAuth2 flow: opens a local HTTP server, returns the authorization URL
/// and the port used by the callback server plus a receiver for the auth code.
pub fn start_oauth_flow(
    config: &OAuthConfig,
) -> Result<(String, u16, mpsc::Receiver<String>), String> {
    let redirect_port = portpicker::pick_unused_port().ok_or("no free port")?;
    let redirect_url = format!("http://localhost:{}/callback", redirect_port);

    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url.clone()).map_err(|e| e.to_string())?)
        .set_token_uri(TokenUrl::new(config.token_url.clone()).map_err(|e| e.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url.clone()).map_err(|e| e.to_string())?);

    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    for scope in &config.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }
    if config.provider == "google" {
        auth_request = auth_request.add_extra_param("access_type", "offline");
        auth_request = auth_request.add_extra_param("prompt", "consent");
    }
    let (auth_url, _csrf_token) = auth_request.url();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let server =
            tiny_http::Server::http(format!("127.0.0.1:{}", redirect_port)).unwrap();
        if let Ok(request) = server.recv() {
            let url = request.url().to_string();
            if let Some(code) = extract_code_from_url(&url) {
                let response = tiny_http::Response::from_string(
                    "<html><body><h1>Authentication successful!</h1>\
                     <p>You can close this window and return to ScouterMail.</p></body></html>",
                )
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
                        .unwrap(),
                );
                let _ = request.respond(response);
                let _ = tx.send(code);
            } else {
                let response = tiny_http::Response::from_string(
                    "Authentication failed. Missing code parameter.",
                );
                let _ = request.respond(response);
            }
        }
    });

    Ok((auth_url.to_string(), redirect_port, rx))
}

fn extract_code_from_url(url: &str) -> Option<String> {
    let query = url.split('?').nth(1)?;
    for param in query.split('&') {
        let mut parts = param.splitn(2, '=');
        if parts.next()? == "code" {
            return Some(parts.next()?.to_string());
        }
    }
    None
}

/// Exchange an authorization code for tokens.
pub async fn exchange_code(
    config: &OAuthConfig,
    code: &str,
    redirect_port: u16,
) -> Result<OAuthTokens, String> {
    let redirect_url = format!("http://localhost:{}/callback", redirect_port);

    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url.clone()).map_err(|e| e.to_string())?)
        .set_token_uri(TokenUrl::new(config.token_url.clone()).map_err(|e| e.to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url).map_err(|e| e.to_string())?);

    let http_client = reqwest::Client::new();

    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(&http_client)
        .await
        .map_err(|e| format!("token exchange failed: {}", e))?;

    let access_token = token_result.access_token().secret().clone();
    let refresh_token = token_result.refresh_token().map(|t| t.secret().clone());
    let expires_at = token_result
        .expires_in()
        .map(|d| chrono::Utc::now().timestamp() + d.as_secs() as i64);

    Ok(OAuthTokens {
        access_token,
        refresh_token,
        expires_at,
    })
}

/// Refresh an expired access token.
pub async fn refresh_access_token(
    config: &OAuthConfig,
    refresh_token: &str,
) -> Result<OAuthTokens, String> {
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url.clone()).map_err(|e| e.to_string())?)
        .set_token_uri(TokenUrl::new(config.token_url.clone()).map_err(|e| e.to_string())?);

    let http_client = reqwest::Client::new();

    let token_result = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
        .request_async(&http_client)
        .await
        .map_err(|e| format!("token refresh failed: {}", e))?;

    let access_token = token_result.access_token().secret().clone();
    let new_refresh = token_result.refresh_token().map(|t| t.secret().clone());
    let expires_at = token_result
        .expires_in()
        .map(|d| chrono::Utc::now().timestamp() + d.as_secs() as i64);

    Ok(OAuthTokens {
        access_token,
        refresh_token: new_refresh.or(Some(refresh_token.to_string())),
        expires_at,
    })
}

/// Build the XOAUTH2 SASL string for IMAP/SMTP authentication.
pub fn build_xoauth2_string(email: &str, access_token: &str) -> String {
    let auth_string = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
    base64::engine::general_purpose::STANDARD.encode(auth_string.as_bytes())
}
