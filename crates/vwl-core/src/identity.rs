use crate::error::{Result, VwlError};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone)]
pub struct NodeKeypair {
    signing_key: SigningKey,
}

impl NodeKeypair {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self { signing_key }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        Self { signing_key }
    }

    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|e| VwlError::InvalidTask(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(VwlError::InvalidTask(
                "Invalid private key length (expected 32 bytes)".to_string(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self::from_bytes(&arr))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key().to_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> String {
        let sig = self.signing_key.sign(message);
        hex::encode(sig.to_bytes())
    }
}

pub fn verify_signature(pubkey_hex: &str, message: &[u8], signature_hex: &str) -> Result<bool> {
    let pub_bytes =
        hex::decode(pubkey_hex).map_err(|e| VwlError::InvalidTask(format!("Bad pubkey: {e}")))?;
    if pub_bytes.len() != 32 {
        return Err(VwlError::InvalidTask(
            "Pubkey must be exactly 32 bytes".to_string(),
        ));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&pub_bytes);
    let verifying_key = VerifyingKey::from_bytes(&arr)
        .map_err(|e| VwlError::InvalidTask(format!("Invalid verifying key: {e}")))?;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| VwlError::InvalidTask(format!("Bad signature: {e}")))?;
    if sig_bytes.len() != 64 {
        return Err(VwlError::InvalidTask(
            "Signature must be exactly 64 bytes".to_string(),
        ));
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    Ok(verifying_key.verify(message, &signature).is_ok())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeCredential {
    pub node_id: String,
    pub public_key: String,
    pub role: String,
    pub display_name: String,
    pub created_at: u64,
    pub signature: String,
}

impl NodeCredential {
    pub fn new(keypair: &NodeKeypair, role: &str, display_name: &str, created_at: u64) -> Self {
        let public_key = keypair.public_key_hex();
        let node_id = format!("node-ed25519-{}", &public_key[..12]);
        let unsigned_payload = format!(
            "node_id:{node_id}|pubkey:{public_key}|role:{role}|name:{display_name}|ts:{created_at}"
        );
        let signature = keypair.sign_message(unsigned_payload.as_bytes());

        Self {
            node_id,
            public_key,
            role: role.to_string(),
            display_name: display_name.to_string(),
            created_at,
            signature,
        }
    }

    pub fn verify(&self) -> Result<bool> {
        let unsigned_payload = format!(
            "node_id:{}|pubkey:{}|role:{}|name:{}|ts:{}",
            self.node_id, self.public_key, self.role, self.display_name, self.created_at
        );
        verify_signature(
            &self.public_key,
            unsigned_payload.as_bytes(),
            &self.signature,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCredential {
    pub project_id: String,
    pub owner_public_key: String,
    pub maintainers: Vec<String>,
    pub created_at: u64,
    pub signature: String,
}

impl ProjectCredential {
    pub fn new(
        owner_keypair: &NodeKeypair,
        project_id: &str,
        maintainers: Vec<String>,
        created_at: u64,
    ) -> Self {
        let owner_public_key = owner_keypair.public_key_hex();
        let unsigned = format!(
            "project_id:{}|owner:{}|maint:{}|ts:{}",
            project_id,
            owner_public_key,
            maintainers.join(","),
            created_at
        );
        let signature = owner_keypair.sign_message(unsigned.as_bytes());

        Self {
            project_id: project_id.to_string(),
            owner_public_key,
            maintainers,
            created_at,
            signature,
        }
    }

    pub fn verify(&self) -> Result<bool> {
        let unsigned = format!(
            "project_id:{}|owner:{}|maint:{}|ts:{}",
            self.project_id,
            self.owner_public_key,
            self.maintainers.join(","),
            self.created_at
        );
        verify_signature(&self.owner_public_key, unsigned.as_bytes(), &self.signature)
    }

    pub fn is_authorized_maintainer(&self, public_key: &str) -> bool {
        self.owner_public_key == public_key || self.maintainers.iter().any(|m| m == public_key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedObject<T> {
    pub payload: T,
    pub signer_public_key: String,
    pub signature: String,
    pub timestamp: u64,
}

impl<T: Serialize + DeserializeOwned> SignedObject<T> {
    pub fn new(keypair: &NodeKeypair, payload: T, timestamp: u64) -> Result<Self> {
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        let sign_target = format!("payload:{payload_json}|ts:{timestamp}");
        let signature = keypair.sign_message(sign_target.as_bytes());

        Ok(Self {
            payload,
            signer_public_key: keypair.public_key_hex(),
            signature,
            timestamp,
        })
    }

    pub fn verify(&self) -> Result<bool> {
        let payload_json = serde_json::to_string(&self.payload)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        let sign_target = format!("payload:{}|ts:{}", payload_json, self.timestamp);
        verify_signature(
            &self.signer_public_key,
            sign_target.as_bytes(),
            &self.signature,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_keypair_generation_and_signing() {
        let keypair = NodeKeypair::generate();
        let message = b"VWL immutable hypothesis unit";
        let sig = keypair.sign_message(message);

        let is_valid = verify_signature(&keypair.public_key_hex(), message, &sig).unwrap();
        assert!(is_valid);

        let corrupted =
            verify_signature(&keypair.public_key_hex(), b"tampered message", &sig).unwrap();
        assert!(!corrupted);
    }

    #[test]
    fn test_node_credential_lifecycle() {
        let keypair = NodeKeypair::generate();
        let cred = NodeCredential::new(&keypair, "developer", "Gemini Node", 1700000000);

        assert!(cred.node_id.starts_with("node-ed25519-"));
        assert!(cred.verify().unwrap());

        // Tamper test
        let mut tampered = cred.clone();
        tampered.role = "compromised-role".to_string();
        assert!(!tampered.verify().unwrap());
    }

    #[test]
    fn test_project_credential_and_membership() {
        let owner = NodeKeypair::generate();
        let member1 = NodeKeypair::generate();
        let outsider = NodeKeypair::generate();

        let proj = ProjectCredential::new(
            &owner,
            "horia-ai/vwl",
            vec![member1.public_key_hex()],
            1700000000,
        );

        assert!(proj.verify().unwrap());
        assert!(proj.is_authorized_maintainer(&owner.public_key_hex()));
        assert!(proj.is_authorized_maintainer(&member1.public_key_hex()));
        assert!(!proj.is_authorized_maintainer(&outsider.public_key_hex()));
    }

    #[test]
    fn test_signed_object_wrapper() {
        let keypair = NodeKeypair::generate();
        let payload = vec!["TASK-001".to_string(), "hash_abc".to_string()];
        let signed = SignedObject::new(&keypair, payload, 1700000000).unwrap();

        assert!(signed.verify().unwrap());

        let mut tampered = signed.clone();
        tampered.payload.push("malicious_addition".to_string());
        assert!(!tampered.verify().unwrap());
    }
}
