use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::Aes256Gcm;
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

const KEY_ENV: &str = "APP_ENCRYPTION_KEY";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const PAYLOAD_VERSION: u8 = 1;

pub type EncryptionResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn encrypt_string(plaintext: &str) -> EncryptionResult<String> {
    let key = encryption_key_from_env()?;
    encrypt_with_key(plaintext, &key)
}

pub fn decrypt_string(ciphertext: &str) -> EncryptionResult<String> {
    let key = encryption_key_from_env()?;
    decrypt_with_key(ciphertext, &key)
}

fn encryption_key_from_env() -> EncryptionResult<[u8; KEY_LEN]> {
    let value = std::env::var(KEY_ENV).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{KEY_ENV} must be set to a 32-byte key or base64-encoded 32-byte key"),
        )
    })?;

    parse_encryption_key(&value)
}

fn parse_encryption_key(value: &str) -> EncryptionResult<[u8; KEY_LEN]> {
    let trimmed = value.trim();
    if trimmed.as_bytes().len() == KEY_LEN {
        return trimmed
            .as_bytes()
            .try_into()
            .map_err(|_| invalid_key_error().into());
    }

    let decoded = general_purpose::STANDARD
        .decode(trimmed)
        .map_err(|_| invalid_key_error())?;

    decoded
        .as_slice()
        .try_into()
        .map_err(|_| invalid_key_error().into())
}

fn encrypt_with_key(plaintext: &str, key: &[u8; KEY_LEN]) -> EncryptionResult<String> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let uuid_bytes = Uuid::new_v4().into_bytes();
    let mut nonce_bytes = [0; NONCE_LEN];
    nonce_bytes.copy_from_slice(&uuid_bytes[..NONCE_LEN]);
    let nonce = aes_gcm::Nonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| encryption_error("invalid nonce"))?;
    let encrypted = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| encryption_error("encryption failed"))?;

    let mut payload = Vec::with_capacity(1 + NONCE_LEN + encrypted.len());
    payload.push(PAYLOAD_VERSION);
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&encrypted);

    Ok(general_purpose::STANDARD.encode(payload))
}

fn decrypt_with_key(ciphertext: &str, key: &[u8; KEY_LEN]) -> EncryptionResult<String> {
    let payload = general_purpose::STANDARD.decode(ciphertext.trim())?;
    if payload.len() <= 1 + NONCE_LEN {
        return Err(encryption_error("encrypted payload is too short").into());
    }

    if payload[0] != PAYLOAD_VERSION {
        return Err(encryption_error("unsupported encrypted payload version").into());
    }

    let nonce = aes_gcm::Nonce::try_from(&payload[1..1 + NONCE_LEN])
        .map_err(|_| encryption_error("invalid nonce"))?;
    let encrypted = &payload[1 + NONCE_LEN..];
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let decrypted = cipher
        .decrypt(&nonce, encrypted)
        .map_err(|_| encryption_error("decryption failed"))?;

    String::from_utf8(decrypted).map_err(|e| e.into())
}

fn invalid_key_error() -> std::io::Error {
    encryption_error("APP_ENCRYPTION_KEY must be 32 raw bytes or base64-encoded 32 bytes")
}

fn encryption_error(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; KEY_LEN] {
        *b"0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn encrypts_and_decrypts_string() {
        let plaintext = "future-provider-access-token";

        let encrypted = encrypt_with_key(plaintext, &test_key()).unwrap();
        let decrypted = decrypt_with_key(&encrypted, &test_key()).unwrap();

        assert_ne!(encrypted, plaintext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encryption_uses_a_unique_nonce() {
        let plaintext = "same-token";

        let first = encrypt_with_key(plaintext, &test_key()).unwrap();
        let second = encrypt_with_key(plaintext, &test_key()).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn parses_base64_encoded_key() {
        let encoded = general_purpose::STANDARD.encode(test_key());

        assert_eq!(parse_encryption_key(&encoded).unwrap(), test_key());
    }

    #[test]
    fn public_functions_read_key_from_environment() {
        let previous_value = std::env::var(KEY_ENV).ok();
        let encoded = general_purpose::STANDARD.encode(test_key());
        std::env::set_var(KEY_ENV, encoded);

        let encrypted = encrypt_string("token-from-env").unwrap();
        let decrypted = decrypt_string(&encrypted).unwrap();

        if let Some(value) = previous_value {
            std::env::set_var(KEY_ENV, value);
        } else {
            std::env::remove_var(KEY_ENV);
        }
        assert_eq!(decrypted, "token-from-env");
    }

    #[test]
    fn rejects_invalid_key_length() {
        let error = parse_encryption_key("short").unwrap_err();

        assert!(error.to_string().contains("APP_ENCRYPTION_KEY"));
    }
}
