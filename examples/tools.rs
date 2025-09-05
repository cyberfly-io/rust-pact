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
