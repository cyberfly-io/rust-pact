# rust-pact

## Example Usage

```rust
use rust_pact::crypto;
use rust_pact::utils::KeyPair;
use rust_pact::api;
use rust_pact::lang;
use rust_pact::simple;
use rust_pact::fetch;
use serde_json::json;

fn main() {
	// Generate a key pair
	let (public_key, secret_key) = crypto::gen_key_pair();
	let kp = KeyPair { public_key, secret_key, clist: None };

	// Prepare meta and cap
	let meta = lang::mk_meta("sender", "0", 0.00001, 1000, 1234567890, 600);
	let cap = lang::mk_cap("role", "desc", "cap.name", vec![json!("arg1"), json!("arg2")]);

	// Prepare exec command
	let pact_code = "(free.my-module.my-func arg1 arg2)";
	let env_data = json!({"key": "value"});
	let key_pairs = vec![kp.clone()];
	let exec_cmd = simple::exec::prepare_exec_cmd(
		pact_code,
		env_data,
		meta.clone(),
		Some("testnet04".to_string()),
		None,
		key_pairs.clone()
	);
	println!("Exec Command: {}", exec_cmd);
}
```

For more examples, see the `examples/` directory.

## Tools Usage Example

```rust
use rust_pact::{tools, gen_key_pair};
use rust_pact::utils::KeyPair;

// Run with: cargo run --example tools
// To perform real network calls set env var RUN_LIVE=1 (these hit public Chainweb endpoints)
fn main() {
	// WARNING: Hard‑coding real secret keys is unsafe. Use env vars or a secure store in production.
	let key_pair = KeyPair {
		public_key: "10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3".into(),
		secret_key: "18d3a823139cf60cab0b738e7605bb9e4a2f3ff245c270fa55d197f9b3c4c004".into(),
		clist: None
	};

	let token_address = "coin"; // same as Python call
	let sender_account = "k:10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3";
	let receiver_account = "k:03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
	let receiver_public_key = "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
	let amount = 2.0_f64;
	let chain_id = "1";
	let network_id = "testnet04"; // matches your Python call

	let res = tools::token_transfer(
		token_address,
		sender_account,
		receiver_account,
		receiver_public_key,
		amount,
		key_pair,
		chain_id,
		network_id
	);

	println!("Transfer result: {}", res);
}
```


cargo build
cargo run --example basic