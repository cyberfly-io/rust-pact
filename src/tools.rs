use crate::{lang, fetch};
use serde_json::{json, Value};
use crate::utils::KeyPair;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::time::Duration;

/// Configuration for the full cross-chain transfer helper.
/// Values can be sourced from environment variables (see Default impl) so callers
/// can tune behavior without code changes.
///
/// Env vars (all optional):
///  XCHAIN_ATTEMPTS            -> attempts for initial tx polling (default 30, 0 = infinite)
///  XCHAIN_INTERVAL_MS         -> interval ms between initial tx polls (default 5000)
///  XCHAIN_POST_CONFIRM_WAIT_MS-> delay after tx success before requesting SPV (default 10000)
///  XCHAIN_SPV_ATTEMPTS        -> attempts for SPV polling (default 0 = infinite, >0 = use limit)
///  XCHAIN_SPV_INTERVAL_MS     -> interval ms between SPV polls (default 5000)
///  XCHAIN_FINAL_ATTEMPTS      -> attempts for final continuation tx poll (default 30, 0 = infinite)
///  XCHAIN_FINAL_INTERVAL_MS   -> interval ms between final polls (default 5000)
///  XCHAIN_VERBOSE             -> if set (any value), emit diagnostic prints
///  XCHAIN_MAX_TOTAL_TIME_MS   -> max total time for all polling combined (default 360000 = 6min, 0 = no limit)
#[derive(Debug, Clone)]
pub struct CrossChainConfig {
    pub attempts_tx: u32,
    pub interval_tx_ms: u64,
    pub post_confirm_wait_ms: u64,
    pub attempts_spv: u32,
    pub interval_spv_ms: u64,
    pub attempts_final: u32,
    pub interval_final_ms: u64,
    pub verbose: bool,
    pub max_total_time_ms: u64,
}

impl Default for CrossChainConfig {
    fn default() -> Self {
        use std::env;
        let attempts_tx = env::var("XCHAIN_ATTEMPTS").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
        let interval_tx_ms = env::var("XCHAIN_INTERVAL_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
        let post_confirm_wait_ms = env::var("XCHAIN_POST_CONFIRM_WAIT_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(10000);
        let attempts_spv = env::var("XCHAIN_SPV_ATTEMPTS").ok().and_then(|v| v.parse().ok()).unwrap_or(0); // Default to infinite for SPV
        let interval_spv_ms = env::var("XCHAIN_SPV_INTERVAL_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
        let attempts_final = env::var("XCHAIN_FINAL_ATTEMPTS").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
        let interval_final_ms = env::var("XCHAIN_FINAL_INTERVAL_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
        let verbose = env::var("XCHAIN_VERBOSE").is_ok();
        let max_total_time_ms = env::var("XCHAIN_MAX_TOTAL_TIME_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(360000); // 6 minutes default to ensure full 5min wait
        Self { attempts_tx, interval_tx_ms, post_confirm_wait_ms, attempts_spv, interval_spv_ms, attempts_final, interval_final_ms, verbose, max_total_time_ms }
    }
}

pub fn get_api_host(network_id: &str, chain_id: &str) -> String {
    match network_id {
        // Correct mapping: networkId must match chainweb path segment
        "mainnet01" => format!("https://api.chainweb.com/chainweb/0.0/mainnet01/chain/{}/pact", chain_id),
        "testnet04" => format!("https://api.testnet.chainweb.com/chainweb/0.0/testnet04/chain/{}/pact", chain_id),
        other => panic!("Unsupported network_id: {}", other)
    }
}

/// Retrieve SPV proof for cross-chain transfer, matching JavaScript reference pattern
/// Simplified polling loop that directly queries SPV without pre-checking transaction status
pub fn poll_create_spv(request_key: &str, source_chain_id: &str, target_chain_id: &str, network_id: &str, max_attempts: Option<u32>, interval_ms: u64) -> Result<String, String> {
    let api_host = get_api_host(network_id, source_chain_id); // SPV retrieved from source chain
    let verbose = std::env::var("XCHAIN_VERBOSE").is_ok();
    let spv_cmd = json!({"requestKey": request_key, "targetChainId": target_chain_id});
    let max_attempts = max_attempts.unwrap_or(0); // 0 = infinite
    let mut attempt = 0;
    // Optional global timeout via env var (milliseconds)
    let max_total_time_ms: u64 = std::env::var("XCHAIN_MAX_TOTAL_TIME_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let start_time = SystemTime::now();

    // Match JavaScript reference: simple do-while loop
    loop {
        attempt += 1;
        if verbose {
            println!("[xchain] spv attempt {} (source chain {}, target chain {})", attempt, source_chain_id, target_chain_id);
        }

        // Check global timeout if configured
        if max_total_time_ms > 0 {
            if let Ok(elapsed) = start_time.elapsed() {
                if elapsed.as_millis() as u64 >= max_total_time_ms {
                    return Err(format!("timeout after {}ms obtaining SPV proof", max_total_time_ms));
                }
            }
        }

        // Match JavaScript reference: directly query SPV without pre-checking transaction status
        let spv_res = fetch::spv(&spv_cmd, &api_host);
        if verbose {
            println!("[xchain]   polled SPV on source chain {} -> keys: {:?}", source_chain_id, spv_res.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        }

        // Handle SPV response matching JavaScript reference logic
        let proof_opt = spv_res.get("proof").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| spv_res.get("body").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .or_else(|| spv_res.as_str().map(|s| s.to_string()));

        if let Some(proof) = proof_opt {
            // Match JavaScript reference: treat certain messages as not-ready
            let not_ready = proof.contains("SPV target not reachable")
                || proof.contains("Transaction hash not found")
                || proof.contains(' ');
            if !not_ready && !proof.is_empty() {
                if verbose {
                    println!("[xchain]   SPV proof ready: {}", proof);
                }
                return Ok(proof);
            }
            if verbose && not_ready {
                println!("[xchain]   SPV not ready: {}", proof);
            }
        }

        // Check attempt limit (0 = infinite)
        if max_attempts > 0 && attempt >= max_attempts {
            return Err(format!("Max attempts ({}) reached waiting for SPV proof", max_attempts));
        }

        // Match JavaScript reference: 10-second delay
        sleep(Duration::from_millis(interval_ms));
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

pub fn crosschain_transfer(token_address: &str,
                          sender_account: &str,
                          receiver_account: &str,
                          receiver_public_key: &str,
                          amount: f64,
                          mut key_pair: KeyPair,
                          source_chain_id: &str,
                          target_chain_id: &str,
                          network_id: &str) -> Value {
    let api_host = get_api_host(network_id, source_chain_id);

    let code = if token_address != "coin" {
        let parts: Vec<&str> = token_address.split('.').collect();
        if parts.len() != 2 { panic!("token address must be namespace.module"); }
        lang::mk_exp(&format!("{}.transfer-crosschain", parts[1]), Some(parts[0]), vec![
            ("sender_account", json!(sender_account)),
            ("receiver_account", json!(receiver_account)),
            ("receiver_guard", json!("(read-keyset \"ks\")")),
            ("target_chain", json!(target_chain_id)),
            ("amount", json!(amount))
        ])
    } else {
        lang::mk_exp("coin.transfer-crosschain", None, vec![
            ("sender_account", json!(sender_account)),
            ("receiver_account", json!(receiver_account)),
            ("receiver_guard", json!("(read-keyset \"ks\")")),
            ("target_chain", json!(target_chain_id)),
            ("amount", json!(amount))
        ])
    };

    // Add capabilities (GAS + TRANSFER)
    key_pair.clist = Some(vec![
        json!({"name": "coin.GAS", "args": []}),
        json!({"name": format!("{}.TRANSFER_XCHAIN", token_address), "args": [sender_account, receiver_account, amount, target_chain_id]})
    ]);

    let creation_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - 100;
    let meta = lang::mk_meta(&format!("k:{}", key_pair.public_key), source_chain_id, 0.0000001, 60000, creation_time as u64, 15000);

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

pub fn crosschain_complete(pact_id: &str,
                          proof: &str,
                          _receiver_account: &str,
                          receiver_public_key: &str,
                          _amount: f64,
                          mut key_pair: KeyPair,
                          target_chain_id: &str,
                          network_id: &str) -> Value {
    let api_host = get_api_host(network_id, target_chain_id);

    // Add capabilities for completing crosschain transfer
    key_pair.clist = Some(vec![
        json!({"name": "coin.GAS", "args": []})
    ]);

    let creation_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - 100;
    let meta = lang::mk_meta(&format!("k:{}", key_pair.public_key), target_chain_id, 0.0000001, 60000, creation_time as u64, 15000);

    let cmd = json!({
        "type": "cont",
        "pactId": pact_id,
        "rollback": false,
        "step": 1,
        "proof": proof,
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

/// Perform a full cross-chain transfer lifecycle (initiate on source chain, poll, obtain SPV, submit continuation on target chain, poll final).
/// This implementation follows the pattern from the JavaScript Kadena client reference:
/// 1. Submit cross-chain transfer to source chain
/// 2. Listen for transaction confirmation on source chain
/// 3. Poll for SPV proof creation (pollCreateSpv equivalent)
/// 4. Submit continuation transaction to target chain
/// 5. Poll for continuation confirmation on target chain (matching JavaScript pollRequestKey)
///
/// Returns a JSON object aggregating all intermediate artifacts:
/// {
///   "request_key_init": <hash>,
///   "init_result": {...},
///   "init_listen_result": {...},
///   "pact_id": <pact id>,
///   "init_status": "success",
///   "spv_proof": <proof string>,
///   "request_key_complete": <hash>,
///   "complete_result": {...},
///   "final_poll_result": {...},
///   "final_status": "success"
/// }
///
/// Errors (network / parsing issues) are surfaced inline in returned JSON fields; caller should inspect
/// "error" keys. Function is best-effort; if a stage fails it stops early and returns what it has.
pub fn crosschain_transfer_full(token_address: &str,
                                sender_account: &str,
                                receiver_account: &str,
                                receiver_public_key: &str,
                                amount: f64,
                                key_pair: KeyPair,
                                source_chain_id: &str,
                                target_chain_id: &str,
                                network_id: &str,
                                config: Option<CrossChainConfig>) -> Value {
    let cfg = config.unwrap_or_default();
    let mut artifacts = json!({"status": "starting"});
    let start_time = SystemTime::now();

    // Helper function to check if we've exceeded max time
    let should_timeout = |start: &SystemTime| -> bool {
        if cfg.max_total_time_ms == 0 { return false; }
        match start.elapsed() {
            Ok(elapsed) => elapsed.as_millis() as u64 >= cfg.max_total_time_ms,
            Err(_) => false,
        }
    };

    // 1. Initiate
    if cfg.verbose { println!("[xchain] initiating transfer..."); }
    let init_res = crosschain_transfer(token_address, sender_account, receiver_account, receiver_public_key, amount, key_pair.clone(), source_chain_id, target_chain_id, network_id);
    let request_key = init_res.get("requestKeys").and_then(|v| v.as_array()).and_then(|arr| arr.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    artifacts["init_result"] = init_res.clone();
    if request_key.is_none() { artifacts["error"] = json!("missing request key from initiation"); return artifacts; }
    let rk = request_key.unwrap();
    artifacts["request_key_init"] = json!(rk);

    // 2. Poll for initiation transaction success first (faster than listen)
    if cfg.verbose { println!("[xchain] polling for transaction status..."); }
    let poll_req = json!({"requestKeys": [rk.clone()]});
    let source_api = get_api_host(network_id, source_chain_id);
    let mut attempt = 0;
    loop {
        attempt += 1;
        if should_timeout(&start_time) {
            artifacts["error"] = json!(format!("timeout after {}ms waiting for initiation tx", cfg.max_total_time_ms));
            return artifacts;
        }
        let res = fetch::poll(&poll_req, &source_api);
        if cfg.verbose { println!("[xchain] poll init attempt {} -> checking tx status", attempt); }
        
        // Check if transaction is in the result
        if let Some(result_obj) = res.get(&rk).and_then(|v| v.as_object()) {
            if result_obj.get("result").and_then(|r| r.get("status")).and_then(|s| s.as_str()) == Some("success") {
                if cfg.verbose { println!("[xchain] transaction mined successfully!"); }
                break;
            }
        }
        
        if cfg.attempts_tx > 0 && attempt >= cfg.attempts_tx {
            artifacts["error"] = json!(format!("max attempts ({}) reached waiting for initiation tx", cfg.attempts_tx));
            return artifacts;
        }
        sleep(Duration::from_millis(cfg.interval_tx_ms));
    };
    
    // 3. Now use listen to get full transaction details
    if cfg.verbose { println!("[xchain] getting full transaction details..."); }
    let listen_req = json!({"listen": rk.clone()});
    let listen_result = fetch::listen(&listen_req, &source_api);
    if cfg.verbose { println!("[xchain] listen completed -> status: {}", listen_result.get("result").and_then(|r| r.get("status")).and_then(|s| s.as_str()).unwrap_or("unknown")); }
    artifacts["init_listen_result"] = listen_result.clone();

    // 3. Extract pactId
    let pact_id = listen_result.get("continuation")
        .and_then(|c| c.get("pactId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    if pact_id.is_none() { artifacts["error"] = json!("missing pactId in init listen result"); return artifacts; }
    let pact_id = pact_id.unwrap();
    artifacts["pact_id"] = json!(pact_id);

    // 4. Extract status
    let status = listen_result.get("result")
        .and_then(|r| r.get("status"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    if status.is_none() { artifacts["error"] = json!("missing status in init listen result"); return artifacts; }
    let status = status.unwrap();
    artifacts["init_status"] = json!(status.clone());
    if cfg.post_confirm_wait_ms > 0 { if cfg.verbose { println!("[xchain] waiting {} ms before SPV fetch", cfg.post_confirm_wait_ms); } sleep(Duration::from_millis(cfg.post_confirm_wait_ms)); }

    // 5. Obtain SPV proof using pollCreateSpv-like approach
    if cfg.verbose { println!("[xchain] obtaining SPV proof..."); }
    let spv_result = poll_create_spv(&rk, source_chain_id, target_chain_id, network_id, if cfg.attempts_spv > 0 { Some(cfg.attempts_spv) } else { None }, cfg.interval_spv_ms);
    let spv_string = match spv_result {
        Ok(proof) => proof,
        Err(err) => {
            artifacts["error"] = json!(err);
            return artifacts;
        }
    };
    artifacts["spv_proof"] = json!(spv_string);

    // 6. Submit continuation on target chain
    if cfg.verbose { println!("[xchain] submitting continuation on target chain..."); }
    let complete_res = crosschain_complete(&pact_id, &spv_string, receiver_account, receiver_public_key, amount, key_pair.clone(), target_chain_id, network_id);
    let request_key_complete = complete_res.get("requestKeys").and_then(|v| v.as_array()).and_then(|arr| arr.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    artifacts["complete_result"] = complete_res.clone();
    if request_key_complete.is_none() { artifacts["error"] = json!("missing request key from completion step"); return artifacts; }
    let rk_complete = request_key_complete.unwrap();
    artifacts["request_key_complete"] = json!(rk_complete);

    // 7. Poll for final completion (matching JavaScript reference using pollRequestKey)
    let final_poll_req = json!({"requestKeys": [rk_complete.clone()]});
    let target_api = get_api_host(network_id, target_chain_id);
    attempt = 0;
    let final_poll_result = loop {
        attempt += 1;
        if should_timeout(&start_time) {
            artifacts["error"] = json!(format!("timeout after {}ms waiting for final completion", cfg.max_total_time_ms));
            return artifacts;
        }
        let res = fetch::poll(&final_poll_req, &target_api);
        if cfg.verbose { println!("[xchain] poll final attempt {} -> keys: {}", attempt, res.as_object().map(|o| o.len()).unwrap_or(0)); }
        if res.get(&rk_complete).is_some() {
            break res;
        }
        if cfg.attempts_final > 0 && attempt >= cfg.attempts_final {
            artifacts["error"] = json!(format!("max attempts ({}) reached waiting for final completion", cfg.attempts_final));
            return artifacts;
        }
        sleep(Duration::from_millis(cfg.interval_final_ms));
    };

    // 8. Extract final status and validate completion
    let final_result = final_poll_result.get(&rk_complete);
    if let Some(result) = final_result {
        let final_status = result.get("result").and_then(|r| r.get("status")).and_then(|s| s.as_str());
        if final_status != Some("success") {
            // Some nodes may return an error like 'resumePact: pact completed' when re-submitting/confirming
            let maybe_err_msg = result.get("result").and_then(|r| r.get("error")).and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("");
            if !maybe_err_msg.to_lowercase().contains("pact completed") && !maybe_err_msg.to_lowercase().contains("resumepact".to_lowercase().as_str()) {
                artifacts["error"] = json!(format!("final transaction failed with status: {:?}, error: {}", final_status, maybe_err_msg));
                return artifacts;
            }
        }
    } else {
        artifacts["error"] = json!("final transaction result not found in poll response");
        return artifacts;
    }

    artifacts["final_poll_result"] = final_poll_result;
    artifacts["final_status"] = json!("success");
    artifacts["status"] = json!("success");
    artifacts
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