use rust_pact::crypto;
use rust_pact::utils::{KeyPair};
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
        Some(key_pairs.clone())
    );
    println!("Exec Command: {}", exec_cmd);

    // Prepare cont command
    let cont_cmd = simple::cont::prepare_cont_cmd(
        "pact-id-123",
        false,
        1,
        None,
        json!({}),
        meta.clone(),
        Some("testnet04".to_string()),
        None,
        Some(key_pairs.clone())
    );
    println!("Cont Command: {}", cont_cmd);

    // Example: Send command to API (mock URL)
    // let api_host = "https://api.testnet.chainweb.com";
    // let response = fetch::send(&exec_cmd, api_host, false);
    // println!("API Response: {}", response);
}
