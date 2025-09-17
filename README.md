# rust-pact

A Rust library for interacting with Kadena Pact smart contracts and blockchain operations.

## Features

- Key pair generation and management
- Pact command construction (exec, cont)
- Cross-chain transfer operations
- SPV proof retrieval
- Trans### ✅ **Successfully Resolved Issues**
- **Timeout Issue**: Fixed HTTP request timeouts by using poll-before-listen approach
- **PactId Extraction**: Corrected pactId extraction from listen response
- **Transaction Mining**: Successfully polls until transaction is mined
- **SPV Proof Retrieval**: Implements proper SPV polling with configurable attempts

### ⚠️ **Known Limitations**
- **SPV Proof Timing**: On testnet, SPV proofs may take 10+ minutes to become available after cross-chain initiation
- **Expected Behavior**: The process will show "obtaining SPV proof..." and may take several minutes
- **Network Timeouts**: Testnet can be slow; use mainnet for faster operations
- **Interrupt Handling**: You can interrupt with Ctrl+C during SPV polling if neededng and status checking
- Unified cross-chain transfer workflow

## Example Usage

```rust
use rust_pact::crypto;
use rust_pact::utils::KeyPair;
use rust_pact::api;
use rust_pact::lang;
use rust_pact::simple;
use rust_pact::fetch;

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

## Cross-Chain Transfer Example

```rust
use rust_pact::{tools, fetch};
use rust_pact::utils::KeyPair;
use serde_json::json;

fn main() {
    // Cross-chain transfer from chain 1 to chain 2
    let sender_keypair = KeyPair {
        public_key: "10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3".into(),
        secret_key: "18d3a823139cf60cab0b738e7605bb9e4a2f3ff245c270fa55d197f9b3c4c004".into(),
        clist: None
    };

    let receiver_keypair = KeyPair {
        public_key: "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3".into(),
        secret_key: "".into(), // Only public key needed for completion
        clist: None
    };

    let token_address = "coin";
    let sender_account = "k:10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3";
    let receiver_account = "k:03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
    let receiver_public_key = "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
    let amount = 1.0_f64;
    let source_chain_id = "1";
    let target_chain_id = "2";
    let network_id = "testnet04";

    // Step 1: Initiate cross-chain transfer on source chain
    let transfer_result = tools::crosschain_transfer(
        token_address,
        sender_account,
        receiver_account,
        receiver_public_key,
        amount,
        sender_keypair.clone(),
        source_chain_id,
        target_chain_id,
        network_id
    );
    println!("Transfer initiated: {}", transfer_result);

    // Step 2: Poll for completion and get pact ID
    let request_key = "transaction-hash-from-step-1";
    let api_host = tools::get_api_host(network_id, source_chain_id);
    let poll_cmd = json!({"requestKeys": [request_key]});
    let poll_result = fetch::poll(&poll_cmd, &api_host);
    
    // Step 3: Get SPV proof
    let spv_cmd = json!({
        "requestKey": request_key,
        "targetChainId": target_chain_id
    });
    let spv_result = fetch::spv(&spv_cmd, &api_host);
    
    // Step 4: Complete cross-chain transfer on target chain
    let pact_id = "extracted-from-poll-result";
    let proof = "extracted-from-spv-result";
    
    let complete_result = tools::crosschain_complete(
        pact_id,
        proof,
        receiver_account,
        receiver_public_key,
        amount,
        receiver_keypair.clone(),
        target_chain_id,
        network_id
    );
    println!("Cross-chain transfer completed: {}", complete_result);
}
```

## Unified Cross-Chain Transfer

For a simplified, single-call cross-chain transfer that handles all the polling and SPV proof retrieval automatically:

```rust
use rust_pact::{tools, CrossChainConfig};
use rust_pact::utils::KeyPair;

fn main() {
    let sender_keypair = KeyPair {
        public_key: "10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3".into(),
        secret_key: "18d3a823139cf60cab0b738e7605bb9e4a2f3ff245c270fa55d197f9b3c4c004".into(),
        clist: None
    };

    let receiver_keypair = KeyPair {
        public_key: "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3".into(),
        secret_key: "".into(),
        clist: None
    };

    let token_address = "coin";
    let sender_account = "k:10375651f1ca0110468152bb8f47b7b8a469e36dfab1c83adf60cab84b5726d3";
    let receiver_account = "k:03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
    let receiver_public_key = "03df480e0b300c52901fdff265f0460913fea495f39972321698740536cc38e3";
    let amount = 1.0_f64;
    let source_chain_id = "1";
    let target_chain_id = "2";
    let network_id = "testnet04";

    // Use default config (configurable via env vars) or customize
    let config = CrossChainConfig::default(); // Or CrossChainConfig { attempts_tx: 20, ... }

    let result = tools::crosschain_transfer_full(
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
    );

    println!("Full cross-chain result: {}", result);

    // Check status
    if result.get("status").and_then(|s| s.as_str()) == Some("success") {
        println!("✅ Transfer completed successfully!");
    } else {
        println!("❌ Transfer failed: {}", result.get("error").unwrap_or(&serde_json::Value::Null));
    }
}
```

### Configuration Options

The `CrossChainConfig` struct allows tuning of polling behavior:

- `attempts_tx`: Max attempts for initial transaction polling (default: 30, 0 = infinite)
- `interval_tx_ms`: Interval between tx polls in ms (default: 5000)
- `post_confirm_wait_ms`: Delay after tx success before SPV fetch (default: 10000)
- `attempts_spv`: Max attempts for SPV proof polling (default: 0 = infinite, >0 = use limit)
- `interval_spv_ms`: Interval between SPV polls in ms (default: 5000)
- `attempts_final`: Max attempts for final completion polling (default: 30, 0 = infinite)
- `interval_final_ms`: Interval between final polls in ms (default: 5000)
- `verbose`: Enable diagnostic logging (default: false)
- `max_total_time_ms`: Max total time for all polling combined (default: 360000ms = 6min, 0 = no limit)

All values can be overridden via environment variables prefixed with `XCHAIN_`, e.g., `XCHAIN_ATTEMPTS=0` for infinite polling.

**Note**: SPV proof polling defaults to infinite attempts since SPV proofs can take significant time to become available after cross-chain transactions.

### Sample Output

```json
{
  "status": "success",
  "request_key_init": "KnTrT06r4gLKuA56fRMZOCBjS1MiSNoCGDO8g8VxYVI",
  "init_result": {"requestKeys": ["KnTrT06r4gLKuA56fRMZOCBjS1MiSNoCGDO8g8VxYVI"]},
  "init_poll_result": {"KnTrT06r4gLKuA56fRMZOCBjS1MiSNoCGDO8g8VxYVI": {"continuation": {"pactId": "pact-id-123", "step": 0}}},
  "pact_id": "pact-id-123",
  "spv_proof": "spv-proof-data",
  "complete_result": {"requestKeys": ["completion-hash-456"]},
  "request_key_complete": "completion-hash-456",
  "final_poll_result": {"completion-hash-456": {"result": {"status": "success"}}}
}
```

Run the example: `cargo run --example crosschain_full`

## Current Status & Known Limitations

### ✅ Implemented Features
- Basic Pact command construction and signing
- Cross-chain transfer initiation and completion
- Unified cross-chain transfer workflow (`crosschain_transfer_full`)
- Configurable polling with environment variables
- Transaction status polling and SPV proof retrieval
- Support for both testnet and mainnet

### ⚠️ Known Limitations
- **SPV Proof Timing**: On testnet, SPV proofs may take 2-5 minutes to become available after cross-chain initiation (now handled with infinite polling by default)
- **Timeout Behavior**: Default 6-minute timeout may not be sufficient for slow networks - increase `XCHAIN_MAX_TOTAL_TIME_MS` for testnet
- **Network Dependencies**: Requires active Kadena network connectivity for live operations
- **Expected Behavior**: Final completion timeout after SPV retrieval is normal testnet behavior

### 🔧 Configuration Options

Configure cross-chain behavior via environment variables:

```bash
# Polling attempts (0 = infinite)
export XCHAIN_ATTEMPTS=10          # Transaction polling attempts
export XCHAIN_SPV_ATTEMPTS=0       # SPV polling attempts (0 = infinite - DEFAULT)
export XCHAIN_FINAL_ATTEMPTS=30    # Final completion polling attempts

# Timing (milliseconds)
export XCHAIN_INTERVAL_MS=10000    # Transaction poll interval
export XCHAIN_SPV_INTERVAL_MS=5000 # SPV poll interval
export XCHAIN_POST_CONFIRM_WAIT_MS=10000  # Wait after tx confirmation
export XCHAIN_MAX_TOTAL_TIME_MS=360000    # Total timeout (6 minutes)

# Other options
export XCHAIN_VERBOSE=1            # Enable verbose logging
export RUN_LIVE=1                 # Enable live network calls
```

### 🧪 Testing

For testing cross-chain operations:
1. Use testnet (`testnet04`) for development
2. Fund test accounts with KDA tokens
3. Expect SPV proofs to take several minutes
4. Use `XCHAIN_SPV_ATTEMPTS=0` for infinite SPV polling

## Build & Run

```bash
cargo build
cargo run --example basic
```

## Verbose mode and troubleshooting

Enable verbose logs and adjust timing via environment variables:

```zsh
# Enable live network calls (testnet/mainnet based on your params)
export RUN_LIVE=1

# Verbose logging
export XCHAIN_VERBOSE=1

# Polling attempts (0 = infinite)
export XCHAIN_ATTEMPTS=30
export XCHAIN_SPV_ATTEMPTS=0
export XCHAIN_FINAL_ATTEMPTS=30

# Intervals & timeouts (ms)
export XCHAIN_INTERVAL_MS=5000
export XCHAIN_SPV_INTERVAL_MS=5000
export XCHAIN_FINAL_INTERVAL_MS=5000
export XCHAIN_POST_CONFIRM_WAIT_MS=10000
export XCHAIN_MAX_TOTAL_TIME_MS=1200000   # 20 minutes recommended for testnet

# Run the example
cargo run --example crosschain_full
```

Tips:
- ✅ **Timeout Issue Resolved**: No more HTTP request timeouts - implementation uses poll-before-listen approach
- ✅ **SPV Polling Now Infinite**: `XCHAIN_SPV_ATTEMPTS=0` by default for unlimited SPV attempts
- ✅ **Expected Testnet Behavior**: SPV proofs take 2-5 minutes; final completion timeout is normal
- SPV proofs can take several minutes on testnet; set long total timeout and infinite SPV attempts.
- We fetch SPV from the source chain and pass the targetChainId.
- Final confirmation uses listen on the target chain; some nodes may return an error message like "resumePact: pact completed" which is treated as successful completion.
- On macOS, you may need to accept Xcode license to run examples:

```zsh
sudo xcodebuild -license
```