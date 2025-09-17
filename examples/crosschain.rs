use rust_pact::{tools, fetch};
use rust_pact::utils::KeyPair;
use serde_json::json;
use std::time::Duration;
use std::thread;

// Run with: cargo run --example crosschain
// To perform real network calls set env var RUN_LIVE=1 (these hit public Chainweb endpoints)
fn main() {
    // Check if we should make real network calls
    let run_live = std::env::var("RUN_LIVE").unwrap_or_default() == "1";
    // Configurable timing via env vars (fallback to defaults)
    let attempts: u32 = std::env::var("XCHAIN_ATTEMPTS").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
    let spv_attempts: u32 = std::env::var("XCHAIN_SPV_ATTEMPTS").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
    let interval_secs: u64 = std::env::var("XCHAIN_INTERVAL").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let spv_interval_secs: u64 = std::env::var("XCHAIN_SPV_INTERVAL").ok().and_then(|v| v.parse().ok()).unwrap_or(interval_secs);
    let post_confirm_wait: u64 = std::env::var("XCHAIN_POST_CONFIRM_WAIT").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
    
    if run_live {
        println!("🌐 RUN_LIVE=1 detected - will make real network calls to testnet");
        println!("⚠️  WARNING: This will attempt actual blockchain transactions!");
    } else {
        println!("🔒 Demo mode - using mock responses (set RUN_LIVE=1 for real calls)");
    }
    
    // WARNING: Hard‑coding real secret keys is unsafe. Use env vars or a secure store in production.
    let sender_keypair = KeyPair {
        public_key: "10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3".into(),
        secret_key: "18d3a823139cf60cab0b738e7605bb9e4a2f3ff245c270fa55d197f9b3c4c004".into(),
        clist: None
    };

    let receiver_keypair = KeyPair {
        public_key: "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3".into(),
        secret_key: "".into(), // Not needed for completion, only public key
        clist: None
    };

    let token_address = "coin";
    let sender_account = "k:10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3";
    let receiver_account = "k:03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
    let receiver_public_key = "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
    let amount = 1.0_f64;
    let source_chain_id = "1"; // Chain where the transfer originates
    let target_chain_id = "2"; // Chain where the transfer completes
    let network_id = "testnet04";

    println!("=== Cross-Chain Transfer Example ===\n");

    // Step 1: Initiate cross-chain transfer on source chain
    println!("Step 1: Initiating cross-chain transfer from chain {} to chain {}", source_chain_id, target_chain_id);
    println!("Amount: {} {}", amount, token_address);
    println!("From: {}", sender_account);
    println!("To: {}", receiver_account);

    let transfer_result = if run_live {
        tools::crosschain_transfer(
            token_address,
            sender_account,
            receiver_account,
            receiver_public_key,
            amount,
            sender_keypair.clone(),
            source_chain_id,
            target_chain_id,
            network_id
        )
    } else {
        // Mock response for demo
        json!({
            "requestKeys": ["KnTrT06r4gLKuA56fRMZOCBjS1MiSNoCGDO8g8VxYVI"]
        })
    };

    println!("Transfer initiation result: {}", serde_json::to_string_pretty(&transfer_result).unwrap());

    // Extract the request key (transaction hash) for polling
    if let Some(request_keys) = transfer_result.get("requestKeys").and_then(|rks| rks.as_array()) {
        if let Some(request_key) = request_keys.get(0).and_then(|rk| rk.as_str()) {
            println!("\nTransaction hash (request key): {}", request_key);
            
            // Step 2: Poll for transaction completion on source chain
            println!("\nStep 2: Polling for transaction completion on source chain...");
            let api_host = tools::get_api_host(network_id, source_chain_id);
            
            let poll_result = if run_live {
                poll_until_result(request_key, &api_host, attempts, interval_secs)
            } else {
                // Mock response for demo
                json!({
                    request_key: {
                        "result": {
                            "status": "success",
                            "data": "Cross-chain transfer initiated"
                        },
                        "continuation": {
                            "pactId": "mock-pact-id-123",
                            "step": 0
                        }
                    }
                })
            };
            println!("Final poll result: {}", serde_json::to_string_pretty(&poll_result).unwrap());

            // Step 3: Extract pact ID from the poll result
            if let Some(pact_id) = extract_pact_id(&poll_result) {
                println!("\nPact ID for continuation: {}", pact_id);

                // Step 4: Get SPV proof (in real scenario, you would get this from the network)
                println!("\nStep 4: Getting SPV proof for cross-chain completion...");
                if run_live && post_confirm_wait > 0 {
                    println!("Waiting {}s after confirmation before requesting SPV (XCHAIN_POST_CONFIRM_WAIT)", post_confirm_wait);
                    thread::sleep(Duration::from_secs(post_confirm_wait));
                }
                let spv_cmd = json!({
                    "requestKey": request_key,
                    "targetChainId": target_chain_id
                });
                
                let spv_result = if run_live {
                    poll_for_spv_proof(&spv_cmd, &api_host, spv_attempts, spv_interval_secs)
                } else {
                    // Mock SPV proof for demo
                    json!({
                        "proof": "mock-spv-proof-data-for-demo"
                    })
                };
                println!("SPV result: {}", serde_json::to_string_pretty(&spv_result).unwrap());

                // Step 5: Complete cross-chain transfer on target chain
                if let Some(proof) = extract_spv_proof(&spv_result) {
                    println!("\nStep 5: Completing cross-chain transfer on target chain...");
                    
                    let complete_result = if run_live {
                        tools::crosschain_complete(
                            &pact_id,
                            &proof,
                            receiver_account,
                            receiver_public_key,
                            amount,
                            receiver_keypair.clone(),
                            target_chain_id,
                            network_id
                        )
                    } else {
                        // Mock completion result for demo
                        json!({
                            "requestKeys": ["completion-tx-hash-456"],
                            "result": "Cross-chain transfer completed successfully"
                        })
                    };

                    println!("Cross-chain completion result: {}", serde_json::to_string_pretty(&complete_result).unwrap());
                } else {
                    println!("Could not extract SPV proof from result");
                    println!("In a real scenario, you would need to wait for the transaction to be included in a block and then request the SPV proof");
                }
            } else {
                println!("Could not extract pact ID from poll result");
            }
        }
    } else {
        println!("No request keys found in transfer result");
    }

    println!("\n=== Example Notes ===");
    println!("1. This example shows the complete cross-chain transfer flow");
    println!("2. In production, you would need to wait for block confirmations between steps");
    println!("3. SPV proofs are generated by the blockchain network after transactions are finalized");
    println!("4. The receiver keypair only needs the public key for the completion step");
    println!("5. Set RUN_LIVE=1 environment variable to execute against real testnet");
}

/// Poll for transaction result until we get a successful result or timeout
fn poll_until_result(request_key: &str, api_host: &str, max_attempts: u32, interval_seconds: u64) -> serde_json::Value {
    let poll_cmd = json!({
        "requestKeys": [request_key]
    });
    
    for attempt in 1..=max_attempts {
        println!("Polling attempt {}/{}", attempt, max_attempts);
        
        let poll_result = fetch::poll(&poll_cmd, api_host);
        
        // Check if we got a meaningful result
        if has_transaction_result(&poll_result, request_key) {
            println!("✓ Transaction found in poll result!");
            return poll_result;
        }
        
        if attempt < max_attempts {
            println!("No result yet, waiting {} seconds before next attempt...", interval_seconds);
            thread::sleep(Duration::from_secs(interval_seconds));
        }
    }
    
    println!("⚠️  Polling timeout reached after {} attempts", max_attempts);
    json!({})
}

/// Check if the poll result contains actual transaction data
fn has_transaction_result(poll_result: &serde_json::Value, request_key: &str) -> bool {
    if let Some(result_map) = poll_result.as_object() {
        if let Some(tx_result) = result_map.get(request_key) {
            // Check if it's not null and has meaningful data
            return !tx_result.is_null() && tx_result.as_object().map_or(false, |obj| !obj.is_empty());
        }
    }
    false
}

/// Poll for SPV proof until available or timeout
/// Poll for SPV proof until available or timeout. Chainweb may need several block confirmations
/// before an SPV proof is available, especially cross-chain (yield inclusion + target adjacency).
/// Common transient states:
/// - Empty JSON / missing `proof`: proof not ready yet
/// - {"error": ...} network or temporal failure; we retry unless permanent-looking
fn poll_for_spv_proof(spv_cmd: &serde_json::Value, api_host: &str, max_attempts: u32, interval_seconds: u64) -> serde_json::Value {
    for attempt in 1..=max_attempts {
        println!("SPV proof attempt {}/{}", attempt, max_attempts);
        
        let spv_result = fetch::spv(spv_cmd, api_host);

        // Log diagnostic hints when no proof
        if !has_spv_proof(&spv_result) {
            if let Some(err) = spv_result.get("error") {
                println!("(diagnostic) SPV response error field: {}", err);
            } else if spv_result.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                println!("(diagnostic) Empty SPV response – likely not yet indexed. Waiting...");
            } else {
                println!("(diagnostic) SPV response keys: {:?}", spv_result.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            }
        }
        
        // Check if we got a valid SPV proof
        if has_spv_proof(&spv_result) {
            println!("✓ SPV proof obtained!");
            return spv_result;
        }
        
        if attempt < max_attempts {
            println!("SPV proof not ready, waiting {} seconds...", interval_seconds);
            thread::sleep(Duration::from_secs(interval_seconds));
        }
    }
    
    println!("⚠️  SPV proof polling timeout after {} attempts", max_attempts);
    json!({})
}

/// Check if the SPV result contains a valid proof
fn has_spv_proof(spv_result: &serde_json::Value) -> bool {
    spv_result.get("proof")
        .and_then(|p| p.as_str())
        .map_or(false, |proof| !proof.is_empty())
}

fn extract_pact_id(poll_result: &serde_json::Value) -> Option<String> {
    // Try to extract pact ID from poll result
    poll_result
        .as_object()?
        .values()
        .next()?
        .get("continuation")?
        .get("pactId")?
        .as_str()
        .map(|s| s.to_string())
}

fn extract_spv_proof(spv_result: &serde_json::Value) -> Option<String> {
    // Try to extract SPV proof from SPV result
    spv_result
        .get("proof")?
        .as_str()
        .map(|s| s.to_string())
}