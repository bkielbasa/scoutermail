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
