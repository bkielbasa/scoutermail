use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use thiserror::Error;

const SERVICE_NAME: &str = "com.scoutermail.accounts";

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("SecFramework error: {0}")]
    SecFramework(String),
}

impl From<security_framework::base::Error> for KeychainError {
    fn from(e: security_framework::base::Error) -> Self {
        KeychainError::SecFramework(e.to_string())
    }
}

/// Store a password in the macOS Keychain for the given account ID.
pub fn store_password(account_id: &str, password: &str) -> Result<(), KeychainError> {
    set_generic_password(SERVICE_NAME, account_id, password.as_bytes())?;
    Ok(())
}

/// Retrieve a password from the macOS Keychain for the given account ID.
pub fn get_password(account_id: &str) -> Result<String, KeychainError> {
    let bytes = get_generic_password(SERVICE_NAME, account_id)?;
    let password = String::from_utf8(bytes)
        .map_err(|e| KeychainError::SecFramework(format!("invalid UTF-8 in password: {}", e)))?;
    Ok(password)
}

/// Delete a password from the macOS Keychain for the given account ID.
pub fn delete_password(account_id: &str) -> Result<(), KeychainError> {
    delete_generic_password(SERVICE_NAME, account_id)?;
    Ok(())
}
