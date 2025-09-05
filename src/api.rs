// API module: command construction, signing, serialization
// ...to be implemented...
use crate::utils::{pull_check_hashs, pull_sig, mk_signer, KeyPair};
use crate::crypto::{sign, attach_sig};
use serde_json::{json, Value};
use chrono::Utc;

pub fn filter_sig(sig: &Value) -> bool {
	sig.get("sig").is_some()
}

pub fn mk_single_cmd(sigs: &[Value], cmd: &str) -> Value {
	json!({
		"hash": pull_check_hashs(sigs),
		"sigs": sigs.iter().filter(|s| filter_sig(s)).map(|s| pull_sig(s)).collect::<Vec<_>>(),
		"cmd": cmd
	})
}

pub fn prepare_exec_cmd(pact_code: &str, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Vec<KeyPair>) -> Value {
	let signers: Vec<Value> = key_pairs.iter().map(|kp| mk_signer(kp)).collect();
	let cmd_json = json!({
		"networkId": network_id,
		"payload": {
			"exec": {
				"data": env_data,
				"code": pact_code
			}
		},
		"signers": signers,
		"meta": meta,
		"nonce": nonce.unwrap_or_else(|| Utc::now().to_rfc3339())
	});
	let cmd = cmd_json.to_string();
	let kp_json: Vec<Value> = key_pairs.iter().map(|kp| json!({"publicKey": kp.public_key, "secretKey": kp.secret_key})).collect();
	let sigs = attach_sig(&cmd, &kp_json);
	mk_single_cmd(&sigs, &cmd)
}

pub fn prepare_cont_cmd(pact_id: &str, rollback: bool, step: u64, proof: Option<String>, env_data: Value, meta: Value, network_id: Option<String>, nonce: Option<String>, key_pairs: Vec<KeyPair>) -> Value {
	let signers: Vec<Value> = key_pairs.iter().map(|kp| mk_signer(kp)).collect();
	let cmd_json = json!({
		"networkId": network_id,
		"payload": {
			"cont": {
				"proof": proof,
				"pactId": pact_id,
				"rollback": rollback,
				"step": step,
				"data": env_data
			}
		},
		"signers": signers,
		"meta": meta,
		"nonce": nonce.unwrap_or_else(|| Utc::now().to_rfc3339())
	});
	let cmd = cmd_json.to_string();
	let kp_json: Vec<Value> = key_pairs.iter().map(|kp| json!({"publicKey": kp.public_key, "secretKey": kp.secret_key})).collect();
	let sigs = attach_sig(&cmd, &kp_json);
	mk_single_cmd(&sigs, &cmd)
}

pub fn mk_public_send(cmds: Vec<Value>) -> Value {
	json!({"cmds": cmds})
}
