use crate::{lang, fetch};
use serde_json::{json, Value};
use crate::utils::KeyPair;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_api_host(network_id: &str, chain_id: &str) -> String {
    match network_id {
        // Correct mapping: networkId must match chainweb path segment
        "mainnet01" => format!("https://api.chainweb.com/chainweb/0.0/mainnet01/chain/{}/pact", chain_id),
        "testnet04" => format!("https://api.testnet.chainweb.com/chainweb/0.0/testnet04/chain/{}/pact", chain_id),
        other => panic!("Unsupported network_id: {}", other)
    }
}

pub fn token_transfer(token_address: &str,
                      sender_account: &str,
                      receiver_account: &str,
                      receiver_public_key: &str,
                      amount: f64,
                      mut key_pair: KeyPair,
                      chain_id: &str,
                      network_id: &str) -> Value {
    let api_host = get_api_host(network_id, chain_id);

    let code = if token_address != "coin" {
        let parts: Vec<&str> = token_address.split('.').collect();
        if parts.len() != 2 { panic!("token address must be namespace.module"); }
        lang::mk_exp(&format!("{}.transfer-create", parts[1]), Some(parts[0]), vec![
            ("sender_account", json!(sender_account)),
            ("receiver_account", json!(receiver_account)),
            ("keyset", json!("(read-keyset \"ks\")")),
            ("amount", json!(amount))
        ])
    } else {
        lang::mk_exp("coin.transfer-create", None, vec![
            ("sender_account", json!(sender_account)),
            ("receiver_account", json!(receiver_account)),
            ("keyset", json!("(read-keyset \"ks\")")),
            ("amount", json!(amount))
        ])
    };

    // Add capabilities (GAS + TRANSFER)
    key_pair.clist = Some(vec![
        json!({"name": "coin.GAS", "args": []}),
        json!({"name": format!("{}.TRANSFER", token_address), "args": [sender_account, receiver_account, amount]})
    ]);

    let creation_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - 100;
    let meta = lang::mk_meta(&format!("k:{}", key_pair.public_key), chain_id, 0.0000001, 60000, creation_time as u64, 15000);

    let cmd = json!({
        "pactCode": code,
        "envData": {"ks": {"pred": "keys-all", "keys": [receiver_public_key]}},
        "meta": meta,
        "networkId": network_id,
        "nonce": chrono::Utc::now().to_rfc3339(),
        "keyPairs": [json!({
            "publicKey": key_pair.public_key,
            "secretKey": key_pair.secret_key,
            "clist": key_pair.clist
        })]
    });

    fetch::send(&cmd, &api_host, false)
}

pub fn get_contract_code(namespace_dot_module: &str, network_id: &str, chain_id: &str) -> Value {
    let describe_code = format!("(describe-module \"{}\")", namespace_dot_module);
    let creation_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let meta = lang::mk_meta("not real", chain_id, 0.0000001, 60000, creation_time, 5000);
    let cmd = json!({
        "pactCode": describe_code,
        "envData": {},
        "meta": meta,
        "networkId": network_id,
        "nonce": chrono::Utc::now().to_rfc3339(),
        "keyPairs": []
    });
    let api_host = get_api_host(network_id, chain_id);
    let res = fetch::local(&cmd, &api_host);
    res
}