use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::accounts::keychain;
use crate::imap::client::ImapConfig;
use crate::smtp::client::SmtpConfig;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("account not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("keychain error: {0}")]
    Keychain(#[from] keychain::KeychainError),
    #[error("store error: {0}")]
    Store(String),
}

// ---------------------------------------------------------------------------
// Account config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
}

// ---------------------------------------------------------------------------
// Provider defaults
// ---------------------------------------------------------------------------

/// Returns `(imap_host, imap_port, smtp_host, smtp_port)` defaults for known
/// email providers, or `None` if the provider is not recognised.
pub fn provider_defaults(provider: &str) -> Option<(String, u16, String, u16)> {
    match provider.to_lowercase().as_str() {
        "gmail" => Some((
            "imap.gmail.com".to_string(),
            993,
            "smtp.gmail.com".to_string(),
            465,
        )),
        "outlook" | "hotmail" => Some((
            "outlook.office365.com".to_string(),
            993,
            "smtp.office365.com".to_string(),
            587,
        )),
        "yahoo" => Some((
            "imap.mail.yahoo.com".to_string(),
            993,
            "smtp.mail.yahoo.com".to_string(),
            465,
        )),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Account manager
// ---------------------------------------------------------------------------

pub struct AccountManager {
    data_dir: PathBuf,
    accounts: Vec<AccountConfig>,
}

impl AccountManager {
    /// Create a new `AccountManager`, loading existing accounts from
    /// `data_dir/accounts.json` if the file exists.
    pub fn new(data_dir: PathBuf) -> Result<Self, AccountError> {
        let accounts_file = data_dir.join("accounts.json");
        let accounts = if accounts_file.exists() {
            let data = fs::read_to_string(&accounts_file)?;
            serde_json::from_str(&data)?
        } else {
            Vec::new()
        };
        Ok(Self { data_dir, accounts })
    }

    /// Persist the current account list to `accounts.json`.
    pub fn save(&self) -> Result<(), AccountError> {
        fs::create_dir_all(&self.data_dir)?;
        let json = serde_json::to_string_pretty(&self.accounts)?;
        fs::write(self.data_dir.join("accounts.json"), json)?;
        Ok(())
    }

    /// Add an account, storing its password in the macOS Keychain and
    /// creating its data directory.
    pub fn add_account(
        &mut self,
        config: AccountConfig,
        password: &str,
    ) -> Result<(), AccountError> {
        keychain::store_password(&config.id, password)?;

        // Create the per-account data directory.
        let account_dir = self.data_dir.join(&config.id);
        fs::create_dir_all(&account_dir)?;

        self.accounts.push(config);
        self.save()?;
        Ok(())
    }

    /// Remove an account by ID, deleting its Keychain entry.
    pub fn remove_account(&mut self, id: &str) -> Result<(), AccountError> {
        let idx = self
            .accounts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;
        self.accounts.remove(idx);
        // Best-effort keychain deletion — ignore errors if the entry was
        // already removed.
        let _ = keychain::delete_password(id);
        self.save()?;
        Ok(())
    }

    /// Return a reference to all accounts.
    pub fn list_accounts(&self) -> &[AccountConfig] {
        &self.accounts
    }

    /// Look up a single account by ID.
    pub fn get_account(&self, id: &str) -> Result<&AccountConfig, AccountError> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))
    }

    /// Build an `ImapConfig` for the given account, fetching the password
    /// from the Keychain.
    pub fn get_imap_config(&self, id: &str) -> Result<ImapConfig, AccountError> {
        let account = self.get_account(id)?;
        let password = keychain::get_password(id)?;
        Ok(ImapConfig {
            host: account.imap_host.clone(),
            port: account.imap_port,
            username: account.username.clone(),
            password,
        })
    }

    /// Build an `SmtpConfig` for the given account, fetching the password
    /// from the Keychain.
    pub fn get_smtp_config(&self, id: &str) -> Result<SmtpConfig, AccountError> {
        let account = self.get_account(id)?;
        let password = keychain::get_password(id)?;
        Ok(SmtpConfig {
            host: account.smtp_host.clone(),
            port: account.smtp_port,
            username: account.username.clone(),
            password,
        })
    }

    /// Path to the SQLite database for the given account.
    pub fn db_path(&self, id: &str) -> PathBuf {
        self.data_dir.join(id).join("mail.db")
    }

    /// Path to the Tantivy search index directory for the given account.
    pub fn search_index_path(&self, id: &str) -> PathBuf {
        self.data_dir.join(id).join("search_index")
    }
}
