use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// 加密服务
pub struct EncryptionService {
    key: Option<[u8; 32]>,
}

impl EncryptionService {
    /// 创建加密服务
    pub fn new(key_base64: Option<String>) -> Self {
        let key = key_base64.and_then(|k| {
            BASE64.decode(&k).ok().and_then(|bytes| {
                if bytes.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    Some(arr)
                } else {
                    tracing::warn!("加密密钥长度无效，应为 32 字节");
                    None
                }
            })
        });

        Self { key }
    }

    /// 是否启用加密
    pub fn is_enabled(&self) -> bool {
        self.key.is_some()
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &str) -> anyhow::Result<String> {
        let key = self.key.ok_or_else(|| anyhow::anyhow!("加密密钥未配置"))?;

        let cipher = Aes256Gcm::new_from_slice(&key)?;

        // 生成随机 nonce
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("加密失败: {}", e))?;

        // 组合 nonce + ciphertext 并 base64 编码
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted: &str) -> anyhow::Result<String> {
        let key = self.key.ok_or_else(|| anyhow::anyhow!("加密密钥未配置"))?;

        let combined = BASE64.decode(encrypted)?;
        if combined.len() < 12 {
            anyhow::bail!("加密数据格式无效");
        }

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("解密失败: {}", e))?;

        String::from_utf8(plaintext).map_err(Into::into)
    }

    /// 生成新的加密密钥
    pub fn generate_key() -> String {
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        BASE64.encode(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionService::generate_key();
        let service = EncryptionService::new(Some(key));

        let plaintext = "Hello, World!";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_no_key() {
        let service = EncryptionService::new(None);
        assert!(!service.is_enabled());
        assert!(service.encrypt("test").is_err());
    }
}
