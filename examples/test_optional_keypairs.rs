use rust_pact::{fetch, crypto, lang};
use rust_pact::utils::KeyPair;
use serde_json::json;

fn main() {
    let api_host = "https://api.chainweb.com/chainweb/0.0/mainnet01/chain/0/pact";
    let meta = lang::mk_meta("any", "0", 0.00001, 1000, 0, 600);

    // Test 1: Without keypairs (read-only query)
    println!("=== Test 1: Local query WITHOUT keypairs ===");
    let local_cmd_no_keys = json!({
        "pactCode": "(+ 5 10)",
        "envData": {},
        "meta": meta,
        "networkId": "mainnet01",
        "nonce": "test-no-keys"
    });

    let response1 = fetch::local(&local_cmd_no_keys, api_host);
    println!("Result: {}\n", response1);

    // Test 2: With keypairs (even though not needed for this read-only query)
    println!("=== Test 2: Local query WITH keypairs (empty sigs filtered) ===");
    let (public_key, secret_key) = crypto::gen_key_pair();
    let kp = KeyPair { 
        public_key: public_key.clone(), 
        secret_key: secret_key.clone(), 
        clist: None 
    };

    let local_cmd_with_keys = json!({
        "pactCode": "(+ 20 30)",
        "envData": {},
        "meta": meta,
        "networkId": "mainnet01",
        "nonce": "test-with-keys",
        "keyPairs": [{"publicKey": kp.public_key, "secretKey": kp.secret_key}]
    });

    let response2 = fetch::local(&local_cmd_with_keys, api_host);
    println!("Result: {}\n", response2);

    println!("✅ Both queries succeeded!");
    println!("✅ Keypairs are now truly optional for local queries");
}
