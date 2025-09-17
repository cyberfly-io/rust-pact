use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hex;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use serde_json::{json, Value};

pub fn hex_to_bin(hex: &str) -> Vec<u8> {
    hex::decode(hex).unwrap_or_default()
}

pub fn bin_to_hex(bin: &[u8]) -> String {
    hex::encode(bin)
}

pub fn hash_bin(msg: &str) -> Vec<u8> {
    // Match Python: hashlib.blake2b(digest_size=32)
    let mut hasher = Blake2bVar::new(32).expect("blake2b var");
    hasher.update(msg.as_bytes());
    let mut out = vec![0u8; 32];
    hasher.finalize_variable(&mut out).expect("finalize blake2b");
    out
}

pub fn b64_url_encoded_hash(bin: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bin)
}

pub fn gen_key_pair() -> (String, String) {
    // For ed25519-dalek 1.0 we use the older rand_core 0.5 compatible OsRng from ed25519-dalek re-export
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    let secret_key = SecretKey::from_bytes(&seed).expect("32 bytes, within curve order");
    let public_key: PublicKey = (&secret_key).into();
    (hex::encode(public_key.as_bytes()), hex::encode(secret_key.as_bytes()))
}

pub fn restore_key_from_secret(secret: &str) -> (String, String) {
    let secret_bytes = hex::decode(secret).unwrap();
    let secret_key = SecretKey::from_bytes(&secret_bytes).unwrap();
    let public_key = PublicKey::from(&secret_key);
    (hex::encode(public_key.as_bytes()), secret.to_string())
}

pub fn sign(msg: &str, secret: &str) -> (String, String) {
    let secret_bytes = hex::decode(secret).unwrap();
    let secret_key = SecretKey::from_bytes(&secret_bytes).unwrap();
    let public_key: PublicKey = (&secret_key).into();
    let keypair = Keypair { secret: secret_key, public: public_key };
    let hash = hash_bin(msg);
    let hash_b64 = b64_url_encoded_hash(&hash);
    let sig: Signature = keypair.sign(&hash);
    (hash_b64, hex::encode(sig.to_bytes()))
}

pub fn verify(msg: &str, public_key: &str, signature: &str) -> bool {
    let public_bytes = hex::decode(public_key).unwrap();
    let public = PublicKey::from_bytes(&public_bytes).unwrap();
    let sig_bytes = hex::decode(signature).unwrap();
    let sig = Signature::from_bytes(&sig_bytes).unwrap();
    let hash = hash_bin(msg);
    public.verify(&hash, &sig).is_ok()
}

// Mirror Python sign_map: if secretKey present sign, else return hash with null sig
pub fn sign_map(msg: &str, kp: &Value) -> Value {
    let hash_bin_v = hash_bin(msg);
    let hash_b64 = b64_url_encoded_hash(&hash_bin_v);
    let public_key = kp.get("publicKey").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(secret) = kp.get("secretKey").and_then(|v| v.as_str()) {
        let (_h, sig_hex) = sign(msg, secret);
        json!({"hash": hash_b64, "sig": sig_hex, "publicKey": public_key})
    } else {
        json!({"hash": hash_b64, "sig": Value::Null, "publicKey": public_key})
    }
}

// Mirror Python attach_sig: accept array of keypair objects
pub fn attach_sig(msg: &str, kp_array: &[Value]) -> Vec<Value> {
    let hash_bin_v = hash_bin(msg);
    let hash_b64 = b64_url_encoded_hash(&hash_bin_v);
    if kp_array.is_empty() {
        return vec![json!({"hash": hash_b64, "sig": Value::Null})];
    }
    kp_array.iter().map(|kp| sign_map(msg, kp)).collect()
}
