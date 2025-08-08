use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine, engine::general_purpose};
use rand::{RngCore, rng};

/// Encrypts the given plaintext password using AES-256-GCM.
///
/// # Arguments
/// * `plain` - The plaintext password to encrypt.
/// * `key` - The 32-byte (256-bit) encryption key.
///
/// # Returns
/// A tuple of base64 encoded ciphertext and nonce.
///
/// # Errors
/// Returns an error if the key length is not 32 bytes or encryption fails.
pub fn encrypt_password(
    plain: &str,
    key: &[u8],
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Ensure key length is 32 bytes (256 bits)
    if key.len() != 32 {
        return Err("Key length must be 32 bytes".into());
    }

    // Initialize cipher with the key
    let cipher = Aes256Gcm::new_from_slice(key)?;

    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the plaintext and handle possible errors
    let ciphertext = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Return base64 encoded ciphertext and nonce
    Ok((
        general_purpose::STANDARD.encode(ciphertext),
        general_purpose::STANDARD.encode(nonce_bytes),
    ))
}

/// Decrypts the given base64 encoded ciphertext using AES-256-GCM.
///
/// # Arguments
/// * `ciphertext_b64` - Base64 encoded ciphertext.
/// * `nonce_b64` - Base64 encoded nonce.
/// * `key` - The 32-byte (256-bit) encryption key.
///
/// # Returns
/// The decrypted plaintext string.
///
/// # Errors
/// Returns an error if the key length is not 32 bytes, decoding fails, nonce length is invalid, or decryption fails.
pub fn decrypt_password(
    ciphertext_b64: &str,
    nonce_b64: &str,
    key: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    if key.len() != 32 {
        return Err("Key length must be 32 bytes".into());
    }

    let cipher = Aes256Gcm::new_from_slice(key)?;

    // Decode base64 inputs
    let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64)?;
    let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64)?;

    if nonce_bytes.len() != 12 {
        return Err("Nonce length must be 12 bytes".into());
    }

    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt the ciphertext and handle possible errors
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // Convert decrypted bytes to UTF-8 string
    String::from_utf8(plaintext).map_err(|e| e.into())
}

pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if hex_str.len() % 2 != 0 {
        return Err("Hex string length must be even".into());
    }
    let bytes = (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()?;
    Ok(bytes)
}

/// Helper function to generate a random 32-byte (256-bit) key.
///
/// # Returns
/// A randomly generated 32-byte array.
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rng().fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key();
        let password = "MyPassword123";

        let (ciphertext, nonce) = encrypt_password(password, &key).unwrap();
        let decrypted = decrypt_password(&ciphertext, &nonce, &key).unwrap();

        assert_eq!(password, decrypted);
    }
}
