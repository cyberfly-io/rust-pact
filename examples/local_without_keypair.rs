use rust_pact::fetch;
use serde_json::json;

fn main() {
    // Example: Local fetch without keypairs (read-only query)
    let local_cmd = json!({
        "pactCode": "(+ 1 2)",
        "envData": {},
        "meta": {
            "chainId": "0",
            "sender": "any",
            "gasLimit": 1000,
            "gasPrice": 0.00001,
            "ttl": 600,
            "creationTime": 0
        },
        "networkId": "mainnet01",
        "nonce": "test-nonce"
        // Note: no keyPairs field - it's optional!
    });

    let api_host = "https://api.chainweb.com/chainweb/0.0/mainnet01/chain/0/pact";
  
    // Example with query parameters
    println!("\nSending with preflight enabled...");
    let response_with_opts = fetch::local_with_options(
        &local_cmd,
        api_host,
        Some(false),  // preflight
        Some(false)  // signatureVerification
    );
    println!("Response with options: {}", response_with_opts);
}
