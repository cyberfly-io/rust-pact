use rust_pact::{tools, CrossChainConfig};
use rust_pact::utils::KeyPair;
use serde_json::json;

// Run with: cargo run --example crosschain_full
// To perform real network calls set env var RUN_LIVE=1 (these hit public Chainweb endpoints)
// Configure timing via env vars: XCHAIN_ATTEMPTS, XCHAIN_INTERVAL_MS, etc. (see CrossChainConfig docs)
fn main() {
    // Check if we should make real network calls
    let run_live = std::env::var("RUN_LIVE").unwrap_or_default() == "1";

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

    let _receiver_keypair = KeyPair {
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

    println!("=== Unified Cross-Chain Transfer Example ===\n");
    println!("Using crosschain_transfer_full for end-to-end flow");
    println!("Amount: {} {}", amount, token_address);
    println!("From: {} (chain {})", sender_account, source_chain_id);
    println!("To: {} (chain {})", receiver_account, target_chain_id);

    // Use default config (env-driven) or customize
    let config = CrossChainConfig::default();
    println!("Config: attempts_tx={}, interval_tx_ms={}, post_confirm_wait_ms={}, attempts_spv={}, interval_spv_ms={}, verbose={}",
             config.attempts_tx, config.interval_tx_ms, config.post_confirm_wait_ms, config.attempts_spv, config.interval_spv_ms, config.verbose);
    println!("Note: Set attempts to 0 for infinite polling (e.g., XCHAIN_ATTEMPTS=0)");

    let result = if run_live {
        tools::crosschain_transfer_full(
            token_address,
            sender_account,
            receiver_account,
            receiver_public_key,
            amount,
            sender_keypair,
            source_chain_id,
            target_chain_id,
            network_id,
            Some(config)
        )
    } else {
        // Mock result for demo - in real mode, this would be the actual aggregated JSON
        json!({
            "status": "success",
            "request_key_init": "mock-init-key-123",
            "init_result": {"requestKeys": ["mock-init-key-123"]},
            "init_poll_result": {"mock-init-key-123": {"continuation": {"pactId": "mock-pact-id-456", "step": 0}}},
            "pact_id": "mock-pact-id-456",
            "spv_proof": "mock-spv-proof-data",
            "complete_result": {"requestKeys": ["mock-complete-key-789"]},
            "request_key_complete": "mock-complete-key-789",
            "final_poll_result": {"mock-complete-key-789": {"result": {"status": "success"}}}
        })
    };

    println!("\nFull cross-chain result: {}", serde_json::to_string_pretty(&result).unwrap());

    if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
        if status == "success" {
            println!("\n✅ Cross-chain transfer completed successfully!");
        } else {
            println!("\n❌ Cross-chain transfer failed or timed out.");
            if let Some(error) = result.get("error") {
                println!("Error: {}", error);
            }
        }
    }

    println!("\n=== Notes ===");
    println!("1. This example uses the unified crosschain_transfer_full function for simplicity.");
    println!("2. Configure timing and attempts via environment variables (see CrossChainConfig docs).");
    println!("3. In production, ensure proper key management and error handling.");
    println!("4. Set RUN_LIVE=1 to execute real transactions on testnet.");
}