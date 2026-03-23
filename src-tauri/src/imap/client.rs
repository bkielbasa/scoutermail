use async_imap::Authenticator;
use async_native_tls::TlsStream;
use futures::StreamExt;
use thiserror::Error;
use tokio::net::TcpStream;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ImapError {
    #[error("connection error: {0}")]
    Connection(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("IMAP error: {0}")]
    Imap(String),
}

impl From<async_imap::error::Error> for ImapError {
    fn from(e: async_imap::error::Error) -> Self {
        ImapError::Imap(e.to_string())
    }
}

impl From<std::io::Error> for ImapError {
    fn from(e: std::io::Error) -> Self {
        ImapError::Connection(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// An authenticated IMAP session over TLS.
pub type ImapSession = async_imap::Session<TlsStream<TcpStream>>;

/// Configuration needed to connect to an IMAP server.
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

/// Connect to an IMAP server over TLS and authenticate, returning a session.
pub async fn connect(config: &ImapConfig) -> Result<ImapSession, ImapError> {
    // 1. TCP connect
    let addr = format!("{}:{}", config.host, config.port);
    let tcp_stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| ImapError::Connection(format!("TCP connect to {}: {}", addr, e)))?;

    // 2. TLS wrap
    let tls_connector = async_native_tls::TlsConnector::new();
    let tls_stream = tls_connector
        .connect(&config.host, tcp_stream)
        .await
        .map_err(|e| ImapError::Connection(format!("TLS handshake with {}: {}", config.host, e)))?;

    // 3. Create IMAP client and read server greeting
    let mut client = async_imap::Client::new(tls_stream);
    let _greeting = client
        .read_response()
        .await
        .ok_or_else(|| ImapError::Connection("no server greeting received".to_string()))?
        .map_err(|e| ImapError::Connection(format!("error reading greeting: {}", e)))?;

    // 4. Login
    let session = client
        .login(&config.username, &config.password)
        .await
        .map_err(|(e, _client)| ImapError::Auth(format!("login failed: {}", e)))?;

    Ok(session)
}

// ---------------------------------------------------------------------------
// XOAUTH2 authenticator
// ---------------------------------------------------------------------------

/// SASL XOAUTH2 authenticator for async-imap.
struct XOAuth2 {
    response: Vec<u8>,
}

impl XOAuth2 {
    fn new(email: &str, access_token: &str) -> Self {
        let s = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
        Self {
            response: s.into_bytes(),
        }
    }
}

impl Authenticator for XOAuth2 {
    type Response = Vec<u8>;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        self.response.clone()
    }
}

/// Connect to an IMAP server over TLS using XOAUTH2 authentication.
pub async fn connect_xoauth2(
    host: &str,
    port: u16,
    email: &str,
    access_token: &str,
) -> Result<ImapSession, ImapError> {
    let addr = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| ImapError::Connection(format!("TCP connect to {}: {}", addr, e)))?;

    let tls_connector = async_native_tls::TlsConnector::new();
    let tls_stream = tls_connector
        .connect(host, tcp_stream)
        .await
        .map_err(|e| ImapError::Connection(format!("TLS handshake with {}: {}", host, e)))?;

    let mut client = async_imap::Client::new(tls_stream);
    let _greeting = client
        .read_response()
        .await
        .ok_or_else(|| ImapError::Connection("no server greeting received".to_string()))?
        .map_err(|e| ImapError::Connection(format!("error reading greeting: {}", e)))?;

    let auth = XOAuth2::new(email, access_token);
    let session = client
        .authenticate("XOAUTH2", auth)
        .await
        .map_err(|(e, _client)| ImapError::Auth(format!("XOAUTH2 auth failed: {}", e)))?;

    Ok(session)
}

/// Connect to IMAP with XOAUTH2 and retry logic.
pub async fn connect_xoauth2_with_retry(
    host: &str,
    port: u16,
    email: &str,
    access_token: &str,
    max_retries: u32,
) -> Result<ImapSession, ImapError> {
    let mut retries = 0;
    loop {
        match connect_xoauth2(host, port, email, access_token).await {
            Ok(session) => return Ok(session),
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(e);
                }
                let delay = std::time::Duration::from_secs(2u64.pow(retries.min(5)));
                log::warn!(
                    "IMAP XOAUTH2 connection failed (attempt {}/{}): {}, retrying in {:?}",
                    retries,
                    max_retries,
                    e,
                    delay
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Connect to an IMAP server with exponential backoff retry logic.
pub async fn connect_with_retry(config: &ImapConfig, max_retries: u32) -> Result<ImapSession, ImapError> {
    let mut retries = 0;
    loop {
        match connect(config).await {
            Ok(session) => return Ok(session),
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(e);
                }
                let delay = std::time::Duration::from_secs(2u64.pow(retries.min(5)));
                log::warn!("IMAP connection failed (attempt {}/{}): {}, retrying in {:?}", retries, max_retries, e, delay);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Move a message from one folder to another by UID.
/// Tries the IMAP MOVE command (RFC 6851) first, falling back to COPY+DELETE+EXPUNGE.
pub async fn move_message(session: &mut ImapSession, uid: u32, from: &str, to: &str) -> Result<(), ImapError> {
    session.select(from).await?;
    let uid_str = uid.to_string();
    match session.uid_mv(&uid_str, to).await {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fallback: COPY then mark \Deleted then EXPUNGE
            session.uid_copy(&uid_str, to).await?;
            let store_stream = session.uid_store(&uid_str, "+FLAGS (\\Deleted)").await?;
            // Consume the stream to complete the command
            let _: Vec<_> = store_stream.collect::<Vec<_>>().await;
            let expunge_stream = session.expunge().await?;
            let _: Vec<_> = expunge_stream.collect::<Vec<_>>().await;
            Ok(())
        }
    }
}

/// Set flags on a message by UID (replaces all flags).
pub async fn set_flags(session: &mut ImapSession, uid: u32, folder: &str, flags: &str) -> Result<(), ImapError> {
    session.select(folder).await?;
    let uid_str = uid.to_string();
    let store_stream = session.uid_store(&uid_str, &format!("FLAGS ({})", flags)).await?;
    // Consume the stream to complete the command
    let _: Vec<_> = store_stream.collect::<Vec<_>>().await;
    Ok(())
}

/// List all folders (mailboxes) on the server.
pub async fn list_folders(session: &mut ImapSession) -> Result<Vec<String>, ImapError> {
    let names_stream = session.list(Some(""), Some("*")).await?;
    let names: Vec<_> = names_stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .map(|name| name.name().to_string())
        .collect();
    Ok(names)
}
