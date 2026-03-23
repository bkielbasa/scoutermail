use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use thiserror::Error;

static PASSWORDS_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("credential store error: {0}")]
    SecFramework(String),
}

/// Set the file path for password storage. Must be called before any other function.
pub fn init(data_dir: &std::path::Path) {
    let path = data_dir.join("passwords.json");
    *PASSWORDS_PATH.lock().expect("password store mutex poisoned") = Some(path);
}

fn passwords_path() -> PathBuf {
    PASSWORDS_PATH
        .lock()
        .expect("password store mutex poisoned")
        .clone()
        .expect("keychain::init() must be called first")
}

fn load_passwords() -> HashMap<String, String> {
    let path = passwords_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_passwords(passwords: &HashMap<String, String>) -> Result<(), KeychainError> {
    let path = passwords_path();
    let data = serde_json::to_string_pretty(passwords)
        .map_err(|e| KeychainError::SecFramework(e.to_string()))?;
    fs::write(&path, data).map_err(|e| KeychainError::SecFramework(e.to_string()))?;
    Ok(())
}

pub fn store_password(account_id: &str, password: &str) -> Result<(), KeychainError> {
    let mut passwords = load_passwords();
    passwords.insert(account_id.to_string(), password.to_string());
    save_passwords(&passwords)
}

pub fn get_password(account_id: &str) -> Result<String, KeychainError> {
    let passwords = load_passwords();
    passwords
        .get(account_id)
        .cloned()
        .ok_or_else(|| KeychainError::SecFramework(format!("no password for {}", account_id)))
}

pub fn delete_password(account_id: &str) -> Result<(), KeychainError> {
    let mut passwords = load_passwords();
    passwords.remove(account_id);
    save_passwords(&passwords)
}

/// Store OAuth tokens as JSON under `{account_id}_oauth_tokens`.
pub fn store_oauth_tokens(
    account_id: &str,
    tokens: &crate::accounts::oauth::OAuthTokens,
) -> Result<(), KeychainError> {
    let key = format!("{}_oauth_tokens", account_id);
    let json =
        serde_json::to_string(tokens).map_err(|e| KeychainError::SecFramework(e.to_string()))?;
    store_password(&key, &json)
}

/// Retrieve OAuth tokens for an account, if they exist.
pub fn get_oauth_tokens(
    account_id: &str,
) -> Option<crate::accounts::oauth::OAuthTokens> {
    let key = format!("{}_oauth_tokens", account_id);
    let json = get_password(&key).ok()?;
    serde_json::from_str(&json).ok()
}

/// Delete OAuth tokens for an account.
pub fn delete_oauth_tokens(account_id: &str) -> Result<(), KeychainError> {
    let key = format!("{}_oauth_tokens", account_id);
    delete_password(&key)
}
