#!/bin/bash

# Create project directory and navigate into it
mkdir -p crypto-workspace
cd crypto-workspace

# Create root Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crypto-lib",
    "crypto-wasm"
]
EOF

# Create crypto-lib structure
mkdir -p crypto-lib/src

# Create crypto-lib/Cargo.toml
cat > crypto-lib/Cargo.toml << 'EOF'
[package]
name = "crypto-lib"
version = "0.1.0"
edition = "2021"

[dependencies]
k256 = { version = "0.13", features = ["ecdsa", "arithmetic", "serde"] }
rand_core = { version = "0.6", features = ["std"] }
hex = "0.4"
EOF

# Create crypto-lib/src/lib.rs
cat > crypto-lib/src/lib.rs << 'EOF'
use k256::{
    ecdsa::{SigningKey, VerifyingKey, Signature, signature::Signer, signature::Verifier},
    SecretKey, PublicKey,
};
use rand_core::OsRng;

pub struct CryptoKeys {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl CryptoKeys {
    pub fn new() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn from_private_key(private_key_bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let signing_key = SigningKey::from_bytes(private_key_bytes)?;
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    pub fn get_public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    pub fn get_private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    pub fn sign_message(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
        self.verifying_key.verify(message, signature).is_ok()
    }

    pub fn derive_shared_secret(&self, public_key_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let public_key = PublicKey::from_sec1_bytes(public_key_bytes)?;
        let secret_key = SecretKey::from_bytes(&self.signing_key.to_bytes())?;
        let shared_secret = secret_key.multiply_point(&public_key.as_affine());
        Ok(shared_secret.to_bytes().to_vec())
    }
}
EOF

# Create crypto-wasm structure
mkdir -p crypto-wasm/src

# Create crypto-wasm/Cargo.toml
cat > crypto-wasm/Cargo.toml << 'EOF'
[package]
name = "crypto-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
crypto-lib = { path = "../crypto-lib" }
wasm-bindgen = "0.2"
hex = "0.4"
k256 = { version = "0.13", features = ["ecdsa"] }
EOF

# Create crypto-wasm/src/lib.rs
cat > crypto-wasm/src/lib.rs << 'EOF'
use wasm_bindgen::prelude::*;
use crypto_lib::CryptoKeys;

#[wasm_bindgen]
pub struct WasmCrypto {
    keys: CryptoKeys,
}

#[wasm_bindgen]
impl WasmCrypto {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            keys: CryptoKeys::new(),
        }
    }

    #[wasm_bindgen]
    pub fn from_private_key(private_key_hex: &str) -> Result<WasmCrypto, JsValue> {
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid private key hex: {}", e)))?;
        
        let keys = CryptoKeys::from_private_key(&private_key_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(Self { keys })
    }

    #[wasm_bindgen]
    pub fn get_public_key(&self) -> String {
        hex::encode(self.keys.get_public_key_bytes())
    }

    #[wasm_bindgen]
    pub fn get_private_key(&self) -> String {
        hex::encode(self.keys.get_private_key_bytes())
    }

    #[wasm_bindgen]
    pub fn sign_message(&self, message: &[u8]) -> String {
        let signature = self.keys.sign_message(message);
        hex::encode(signature.to_bytes())
    }

    #[wasm_bindgen]
    pub fn verify_signature(&self, message: &[u8], signature_hex: &str) -> Result<bool, JsValue> {
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid signature hex: {}", e)))?;
        
        let signature = k256::ecdsa::Signature::from_bytes(&signature_bytes.try_into()
            .map_err(|_| JsValue::from_str("Invalid signature length"))?)
            .map_err(|e| JsValue::from_str(&format!("Invalid signature: {}", e)))?;
        
        Ok(self.keys.verify_signature(message, &signature))
    }

    #[wasm_bindgen]
    pub fn derive_shared_secret(&self, public_key_hex: &str) -> Result<String, JsValue> {
        let public_key_bytes = hex::decode(public_key_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid public key hex: {}", e)))?;
        
        let shared_secret = self.keys.derive_shared_secret(&public_key_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(hex::encode(shared_secret))
    }
}
EOF

# Create a .gitignore file
cat > .gitignore << 'EOF'
/target
Cargo.lock
pkg/
EOF
