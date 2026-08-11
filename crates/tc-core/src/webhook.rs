use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, VecDeque};

type HmacSha256 = Hmac<Sha256>;

pub fn request_fingerprint(body: &[u8]) -> String {
    hex::encode(Sha256::digest(body))
}

pub fn verify_hmac_sha256(secret: &str, body: &[u8], provided: &str) -> Result<()> {
    let candidate = provided.strip_prefix("sha256=").unwrap_or(provided);
    let signature = hex::decode(candidate).context("webhook HMAC is not valid hexadecimal")?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).context("invalid HMAC secret")?;
    mac.update(body);
    mac.verify_slice(&signature)
        .map_err(|_| anyhow::anyhow!("webhook HMAC verification failed"))
}

pub fn verify_discord_ed25519(
    public_key_hex: &str,
    signature_hex: &str,
    timestamp: &str,
    body: &[u8],
) -> Result<()> {
    let public_key =
        hex::decode(public_key_hex).context("Discord public key is not valid hexadecimal")?;
    let key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| anyhow::anyhow!("Discord public key must be 32 bytes"))?;
    let signature_bytes =
        hex::decode(signature_hex).context("Discord signature is not valid hexadecimal")?;
    let signature =
        Signature::from_slice(&signature_bytes).context("Discord signature must be 64 bytes")?;
    let mut signed = timestamp.as_bytes().to_vec();
    signed.extend_from_slice(body);
    VerifyingKey::from_bytes(&key)
        .context("invalid Discord public key")?
        .verify(&signed, &signature)
        .map_err(|_| anyhow::anyhow!("Discord signature verification failed"))
}

#[derive(Debug)]
pub struct ReplayGuard {
    ids: HashSet<String>,
    order: VecDeque<String>,
    capacity: usize,
}

impl ReplayGuard {
    pub fn new(capacity: usize) -> Self {
        Self {
            ids: HashSet::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }
    /// Returns `true` only for a previously unseen request ID or content fingerprint.
    pub fn accept_once(&mut self, id: impl Into<String>) -> bool {
        let id = id.into();
        if !self.ids.insert(id.clone()) {
            return false;
        }
        self.order.push_back(id);
        while self.order.len() > self.capacity {
            if let Some(expired) = self.order.pop_front() {
                self.ids.remove(&expired);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifies_hmac_and_rejects_replays() {
        let body = br#"{"message":"hello"}"#;
        let mut mac = HmacSha256::new_from_slice(b"secret").unwrap();
        mac.update(body);
        let signature = hex::encode(mac.finalize().into_bytes());
        verify_hmac_sha256("secret", body, &signature).unwrap();
        assert!(verify_hmac_sha256("wrong", body, &signature).is_err());
        let mut guard = ReplayGuard::new(2);
        assert!(guard.accept_once(request_fingerprint(body)));
        assert!(!guard.accept_once(request_fingerprint(body)));
    }
}
