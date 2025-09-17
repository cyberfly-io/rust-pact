pub mod crypto;
pub mod api;
pub mod lang;
pub mod simple;
pub mod fetch;
pub mod utils;
pub mod tools;

pub use crypto::{gen_key_pair, sign, verify, attach_sig, sign_map, b64_url_encoded_hash, hash_bin, hex_to_bin, bin_to_hex};
pub use fetch::{send, listen, poll, local, spv, send_signed, simple_poll_req_from_exec, simple_listen_req_from_exec};
pub use tools::{get_api_host, token_transfer, crosschain_transfer, crosschain_complete, crosschain_transfer_full, CrossChainConfig, poll_create_spv};

use serde_json::Value;
use utils::KeyPair;

/// High-level facade mirroring the original Python Pact class layout.
pub struct Pact;

impl Pact {
	pub fn prepare_exec_cmd(pact_code: &str, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Vec<KeyPair>) -> Value {
		api::prepare_exec_cmd(pact_code, env_data, meta, network_id, nonce, key_pairs)
	}

	pub fn prepare_cont_cmd(pact_id: &str, rollback: bool, step: u64, proof: Option<String>, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Vec<KeyPair>) -> Value {
		api::prepare_cont_cmd(pact_id, rollback, step, proof, env_data, meta, network_id, nonce, key_pairs)
	}

	pub fn send(cmd: &Value, api_host: &str, debug: bool) -> Value { fetch::send(cmd, api_host, debug) }
	pub fn poll(poll_cmd: &Value, api_host: &str) -> Value { fetch::poll(poll_cmd, api_host) }
	pub fn listen(listen_cmd: &Value, api_host: &str) -> Value { fetch::listen(listen_cmd, api_host) }
	pub fn local(local_cmd: &Value, api_host: &str) -> Value { fetch::local(local_cmd, api_host) }
	pub fn spv(spv_cmd: &Value, api_host: &str) -> Value { fetch::spv(spv_cmd, api_host) }
}
