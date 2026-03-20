use std::time::Duration;

use log::info;

use super::client::{ImapError, ImapSession};
use async_imap::extensions::idle::IdleResponse;

// ---------------------------------------------------------------------------
// IDLE support
// ---------------------------------------------------------------------------

/// Enter IMAP IDLE mode on the given folder, waiting for server-side changes.
///
/// This will block (asynchronously) until:
/// - The server reports new data (e.g. new message arrived).
/// - 29 minutes elapse (per RFC 2177 recommendation), at which point the
///   caller should re-issue IDLE.
/// - An error occurs.
///
/// The session is consumed by IDLE and returned on completion so the caller
/// can continue using it.
pub async fn idle_wait(
    session: ImapSession,
    folder: &str,
) -> Result<ImapSession, ImapError> {
    // NOTE: Session::idle() consumes the session; we get it back from handle.done().
    // The folder must already be selected before calling this function,
    // or we select it here for safety.

    // We need a mutable session to select, but idle() takes ownership.
    // So we select first, then hand off to idle.
    let mut session = session;
    session
        .select(folder)
        .await
        .map_err(|e| ImapError::Imap(format!("select {} for idle: {}", folder, e)))?;

    info!("entering IDLE on folder {}", folder);

    // Create the IDLE handle (consumes the session)
    let mut idle_handle = session.idle();

    // Initialize the IDLE command
    idle_handle
        .init()
        .await
        .map_err(|e| ImapError::Imap(format!("idle init: {}", e)))?;

    // Wait for up to 29 minutes (RFC 2177 recommendation)
    let idle_timeout = Duration::from_secs(29 * 60);
    let (idle_future, _stop_source) = idle_handle.wait_with_timeout(idle_timeout);

    match idle_future.await {
        Ok(IdleResponse::NewData(data)) => {
            info!("IDLE got new data: {:?}", data);
        }
        Ok(IdleResponse::Timeout) => {
            info!("IDLE timed out after 29 minutes, will re-issue");
        }
        Ok(IdleResponse::ManualInterrupt) => {
            info!("IDLE was manually interrupted");
        }
        Err(e) => {
            return Err(ImapError::Imap(format!("idle wait error: {}", e)));
        }
    }

    // End IDLE and recover the session
    let session = idle_handle
        .done()
        .await
        .map_err(|e| ImapError::Imap(format!("idle done: {}", e)))?;

    Ok(session)
}
