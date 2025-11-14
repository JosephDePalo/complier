use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};

use sha2::{Digest, Sha256};

use anyhow::Result;

pub fn make_cipher(master_key: &str) -> Aes256Gcm {
    let mut hasher = Sha256::new();
    hasher.update(master_key.as_bytes());
    let key_bytes = hasher.finalize();
    #[allow(deprecated)]
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    Aes256Gcm::new(&key)
}

pub fn encrypt_password(
    cipher: &Aes256Gcm,
    password: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, password.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed password encryption: {}", e))?;
    return Ok((nonce.to_vec(), ciphertext));
}

pub fn decrypt_password(
    cipher: &Aes256Gcm,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
) -> Result<String> {
    #[allow(deprecated)]
    let nonce = Nonce::from_slice(&nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Failed password decryption: {}", e))?;
    let utf8_string = String::from_utf8(plaintext)?;
    Ok(utf8_string)
}
