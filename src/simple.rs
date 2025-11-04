use crate::api::{mk_public_send}; // Removed unused imports
use crate::utils::KeyPair;
use serde_json::Value;

pub mod exec {
    use super::*;
    pub fn prepare_exec_cmd(pact_code: &str, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Option<Vec<KeyPair>>) -> Value {
        crate::api::prepare_exec_cmd(pact_code, env_data, meta, network_id, nonce, key_pairs)
    }

    pub fn simple_exec_command(pact_code: &str, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Option<Vec<KeyPair>>) -> Value {
        let cmd = prepare_exec_cmd(pact_code, env_data, meta, network_id, nonce, key_pairs);
        mk_public_send(vec![cmd])
    }
}

pub mod cont {
    use super::*;
    pub fn prepare_cont_cmd(pact_id: &str, rollback: bool, step: u64, proof: Option<String>, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Option<Vec<KeyPair>>) -> Value {
        crate::api::prepare_cont_cmd(pact_id, rollback, step, proof, env_data, meta, network_id, nonce, key_pairs)
    }

    pub fn simple_cont_command(pact_id: &str, rollback: bool, step: u64, proof: Option<String>, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Option<Vec<KeyPair>>) -> Value {
        let cmd = prepare_cont_cmd(pact_id, rollback, step, proof, env_data, meta, network_id, nonce, key_pairs);
        mk_public_send(vec![cmd])
    }
}
