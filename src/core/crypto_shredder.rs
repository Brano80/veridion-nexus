use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use rand::thread_rng;
use std::collections::HashMap;
use std::sync::Mutex;

/// Immutable encrypted log entry
/// Contains only ciphertext and nonce, no keys
#[derive(Debug, Clone)]
pub struct EncryptedLog {
    pub log_id: String,
    #[allow(dead_code)]
    pub ciphertext: Vec<u8>,
    #[allow(dead_code)]
    pub nonce: Vec<u8>,
}

/// Mutable key store for managing wrapped Data Encryption Keys (DEKs)
pub struct VeridionKeyStore {
    /// Maps log_id -> Wrapped DEK (encrypted with master key)
    keys: Mutex<HashMap<String, Vec<u8>>>,
    /// Master key for wrapping/unwrapping DEKs
    master_key: Vec<u8>,
}

impl VeridionKeyStore {
    /// Create a new VeridionKeyStore by loading the master key from environment
    /// 
    /// # Panics
    /// 
    /// Panics if VERIDION_MASTER_KEY is not set in the environment
    pub fn new() -> Self {
        let master_key_str = std::env::var("VERIDION_MASTER_KEY")
            .expect("VERIDION_MASTER_KEY must be set in environment");
        
        // Convert the master key string to bytes
        // For simplicity, we'll use the first 32 bytes (pad or truncate as needed)
        let mut master_key = master_key_str.as_bytes().to_vec();
        
        // Ensure master key is exactly 32 bytes (256 bits) for AES-256
        if master_key.len() < 32 {
            // Pad with zeros if too short
            master_key.resize(32, 0);
        } else if master_key.len() > 32 {
            // Truncate if too long
            master_key.truncate(32);
        }
        
        Self {
            keys: Mutex::new(HashMap::new()),
            master_key,
        }
    }

    /// Log an event by encrypting the payload and storing the wrapped DEK
    /// 
    /// # Arguments
    /// 
    /// * `payload` - The plaintext data to encrypt
    /// 
    /// # Returns
    /// 
    /// * `EncryptedLog` containing the encrypted data and metadata (no keys)
    pub fn log_event(&self, payload: &str) -> EncryptedLog {
        // Generate a random 256-bit (32-byte) DEK
        let mut dek = vec![0u8; 32];
        thread_rng().fill_bytes(&mut dek);
        
        // Create AES-256-GCM cipher with the DEK
        let cipher = Aes256Gcm::new_from_slice(&dek)
            .expect("Failed to create cipher from DEK");
        
        // Generate a random 96-bit (12-byte) nonce for GCM
        let mut nonce_bytes = vec![0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the payload using the DEK
        let ciphertext = cipher
            .encrypt(nonce, payload.as_bytes())
            .expect("Failed to encrypt payload");
        
        // Wrap (encrypt) the DEK using the master key
        let master_cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .expect("Failed to create master cipher");
        
        // Generate a nonce for wrapping the DEK
        let mut wrap_nonce_bytes = vec![0u8; 12];
        thread_rng().fill_bytes(&mut wrap_nonce_bytes);
        let wrap_nonce = Nonce::from_slice(&wrap_nonce_bytes);
        
        // Encrypt the DEK (wrap it)
        // We'll prepend the wrap_nonce to the wrapped DEK for later decryption
        let wrapped_dek = master_cipher
            .encrypt(wrap_nonce, dek.as_slice())
            .expect("Failed to wrap DEK");
        
        // Combine wrap_nonce and wrapped_dek for storage
        let mut wrapped_dek_with_nonce = wrap_nonce_bytes;
        wrapped_dek_with_nonce.extend_from_slice(&wrapped_dek);
        
        // Generate a unique log_id using random bytes
        let mut log_id_bytes = vec![0u8; 16];
        thread_rng().fill_bytes(&mut log_id_bytes);
        let log_id = format!("log_{}", log_id_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        
        // Store log_id -> Wrapped DEK in the HashMap
        self.keys
            .lock()
            .expect("Failed to acquire lock")
            .insert(log_id.clone(), wrapped_dek_with_nonce);
        
        EncryptedLog {
            log_id,
            ciphertext,
            nonce: nonce_bytes,
        }
    }

    /// Read an event by decrypting using the stored wrapped DEK
    /// 
    /// # Arguments
    /// 
    /// * `log` - The encrypted log entry to decrypt
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` with the decrypted payload
    /// * `Err(String)` with "GDPR_PURGED: Data key destroyed" if the key was shredded
    #[allow(dead_code)]
    pub fn read_event(&self, log: &EncryptedLog) -> Result<String, String> {
        // Look up log.log_id in the HashMap
        let wrapped_dek_with_nonce = self
            .keys
            .lock()
            .expect("Failed to acquire lock")
            .get(&log.log_id)
            .ok_or_else(|| "GDPR_PURGED: Data key destroyed".to_string())?
            .clone();
        
        // Extract the wrap nonce (first 12 bytes) and wrapped DEK (rest)
        let wrap_nonce_bytes = &wrapped_dek_with_nonce[..12];
        let wrapped_dek = &wrapped_dek_with_nonce[12..];
        
        // Decrypt the wrapped DEK using master key
        let master_cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .expect("Failed to create master cipher");
        
        let wrap_nonce = Nonce::from_slice(wrap_nonce_bytes);
        let dek = master_cipher
            .decrypt(wrap_nonce, wrapped_dek)
            .map_err(|_| "Failed to unwrap DEK".to_string())?;
        
        // Decrypt the log.ciphertext using the restored DEK
        let cipher = Aes256Gcm::new_from_slice(&dek)
            .expect("Failed to create cipher from DEK");
        
        let nonce = Nonce::from_slice(&log.nonce);
        let plaintext = cipher
            .decrypt(nonce, log.ciphertext.as_ref())
            .map_err(|_| "Failed to decrypt payload".to_string())?;
        
        // Convert bytes to string
        String::from_utf8(plaintext)
            .map_err(|_| "Invalid UTF-8 in decrypted data".to_string())
    }

    /// Get wrapped DEK for a log entry (for database storage)
    /// 
    /// # Arguments
    /// 
    /// * `log_id` - The log ID
    /// 
    /// # Returns
    /// 
    /// * `Some(Vec<u8>)` with wrapped DEK if exists
    /// * `None` if key was shredded or doesn't exist
    pub fn get_wrapped_dek(&self, log_id: &str) -> Option<Vec<u8>> {
        self.keys
            .lock()
            .expect("Failed to acquire lock")
            .get(log_id)
            .cloned()
    }

    /// Shred (delete) the key for a specific log entry
    /// 
    /// # Arguments
    /// 
    /// * `log_id` - The log ID whose key should be shredded
    pub fn shred_key(&self, log_id: &str) {
        self.keys
            .lock()
            .expect("Failed to acquire lock")
            .remove(log_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle() {
        // Set up test environment
        unsafe {
            std::env::set_var("VERIDION_MASTER_KEY", "test_master_key_32_bytes_long_123456");
        }
        
        let store = VeridionKeyStore::new();
        let payload = "Secret Data";
        
        // Log the event
        let encrypted_log = store.log_event(payload);
        
        // Read it back
        let decrypted = store.read_event(&encrypted_log)
            .expect("Failed to read event");
        
        // Assert it matches
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_shredding() {
        // Set up test environment
        unsafe {
            std::env::set_var("VERIDION_MASTER_KEY", "test_master_key_32_bytes_long_123456");
        }
        
        let store = VeridionKeyStore::new();
        let payload = "User Email";
        
        // Log the event
        let encrypted_log = store.log_event(payload);
        
        // Shred the key
        store.shred_key(&encrypted_log.log_id);
        
        // Attempt to read - should fail with GDPR_PURGED error
        let result = store.read_event(&encrypted_log);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "GDPR_PURGED: Data key destroyed");
    }
}

