use crate::api::{mk_public_send, prepare_exec_cmd, prepare_cont_cmd};
use crate::utils::{unique, get_headers, parse_res, KeyPair};
use serde_json::{Value, json};
use reqwest::blocking::Client;

#[derive(Debug, Clone, Default)]
pub struct LocalOptions {
    pub preflight: Option<bool>,
    pub signature_verification: Option<bool>,
}

fn create_http_client() -> Client {
    Client::new()
}

pub fn simple_poll_req_from_exec(exec_msg: &Value) -> Value {
    let cmds = exec_msg.get("cmds").and_then(|v| v.as_array()).expect("expected key 'cmds' in object");
    let rks: Vec<String> = cmds.iter().map(|cmd| cmd.get("hash").and_then(|h| h.as_str()).expect("malformed object, expected 'hash' key in every cmd").to_string()).collect();
    json!({"requestKeys": unique(&rks)})
}

pub fn simple_listen_req_from_exec(exec_msg: &Value) -> Value {
    let cmds = exec_msg.get("cmds").and_then(|v| v.as_array()).expect("expected key 'cmds' in object");
    let hsh = cmds[0].get("hash").and_then(|h| h.as_str()).expect("malformed object, expected 'hash' key in every cmd").to_string();
    json!({"listen": hsh})
}

fn value_to_keypair(v: &Value) -> KeyPair {
    KeyPair {
        public_key: v.get("publicKey").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        secret_key: v.get("secretKey").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        clist: v.get("clist").and_then(|c| c.as_array()).map(|arr| arr.clone()),
    }
}

pub fn make_prepare_cmd(cmd: &Value) -> Value {
    // Cont command if type == "cont"
    if cmd.get("type").and_then(|v| v.as_str()) == Some("cont") {
        let pact_id = cmd.get("pactId").and_then(|v| v.as_str()).unwrap_or("");
        let rollback = cmd.get("rollback").and_then(|v| v.as_bool()).unwrap_or(false);
        let step = cmd.get("step").and_then(|v| v.as_u64()).unwrap_or(0);
        let proof = cmd.get("proof").and_then(|v| v.as_str()).map(|s| s.to_string());
        let env_data = cmd.get("envData").cloned().unwrap_or(json!({}));
        let meta = cmd.get("meta").cloned().unwrap_or(json!({}));
        let network_id = cmd.get("networkId").and_then(|v| v.as_str()).map(|s| s.to_string());
        let nonce = cmd.get("nonce").and_then(|v| v.as_str()).map(|s| s.to_string());
        let key_pairs: Vec<KeyPair> = cmd.get("keyPairs").and_then(|v| v.as_array()).map(|arr| arr.iter().map(value_to_keypair).collect()).unwrap_or_default();
        return prepare_cont_cmd(pact_id, rollback, step, proof, env_data, meta, network_id, nonce, key_pairs);
    }
    // Exec command default
    let pact_code = cmd.get("pactCode").and_then(|v| v.as_str()).unwrap_or("");
    let env_data = cmd.get("envData").cloned().unwrap_or(json!({}));
    let meta = cmd.get("meta").cloned().unwrap_or(json!({}));
    let network_id = cmd.get("networkId").and_then(|v| v.as_str()).map(|s| s.to_string());
    let nonce = cmd.get("nonce").and_then(|v| v.as_str()).map(|s| s.to_string());
    let key_pairs: Vec<KeyPair> = cmd.get("keyPairs").and_then(|v| v.as_array()).map(|arr| arr.iter().map(value_to_keypair).collect()).unwrap_or_default();
    prepare_exec_cmd(pact_code, env_data, meta, network_id, nonce, key_pairs)
}

pub fn fetch_send_raw(send_cmd: &Value, api_host: &str, debug: bool) -> reqwest::blocking::Response {
    let client = create_http_client();
    // If already in form {"cmds": [...]}, assume pre-prepared
    let prepared_cmds: Vec<Value> = if let Some(arr) = send_cmd.get("cmds").and_then(|v| v.as_array()) {
        arr.to_vec()
    } else if send_cmd.is_array() {
        send_cmd.as_array().unwrap().iter().map(make_prepare_cmd).collect()
    } else {
        vec![make_prepare_cmd(send_cmd)]
    };
    if debug { println!("prepared_cmds: {:?}", prepared_cmds); }
    client.post(&format!("{}/api/v1/send", api_host))
        .json(&mk_public_send(prepared_cmds))
        .headers(get_headers())
        .send()
        .expect("Failed to send request")
}

pub fn send(send_cmd: &Value, api_host: &str, debug: bool) -> Value {
    let res = fetch_send_raw(send_cmd, api_host, debug);
    parse_res(res)
}

pub fn fetch_spv_raw(spv_cmd: &Value, api_host: &str) -> reqwest::blocking::Response {
    let client = create_http_client();
    client.post(&format!("{}/spv", api_host))
        .json(spv_cmd)
        .headers(get_headers())
        .send()
        .expect("Failed to send request")
}

pub fn spv(spv_cmd: &Value, api_host: &str) -> Value {
    let res = fetch_spv_raw(spv_cmd, api_host);
    parse_res(res)
}

pub fn fetch_local_raw(local_cmd: &Value, api_host: &str, options: Option<LocalOptions>) -> reqwest::blocking::Response {
    let client = create_http_client();
    let mut url = format!("{}/api/v1/local", api_host);
    
    // Build query string based on provided options
    if let Some(opts) = options {
        let mut query_params = Vec::new();
        if let Some(pf) = opts.preflight {
            query_params.push(format!("preflight={}", pf));
        }
        if let Some(sv) = opts.signature_verification {
            query_params.push(format!("signatureVerification={}", sv));
        }
        
        if !query_params.is_empty() {
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }
    }
    
    client.post(&url)
        .json(local_cmd)
        .headers(get_headers())
        .send()
        .expect("Failed to send request")
}

// Primary function with options struct
pub fn local_with_opts(local_cmd: &Value, api_host: &str, options: Option<LocalOptions>) -> Value {
    let res = fetch_local_raw(local_cmd, api_host, options);
    parse_res(res)
}

// Default function without options (backward compatible)
pub fn local(local_cmd: &Value, api_host: &str) -> Value {
    local_with_opts(local_cmd, api_host, None)
}

// Convenience function for individual parameters
pub fn local_with_options(local_cmd: &Value, api_host: &str, preflight: Option<bool>, signature_verification: Option<bool>) -> Value {
    let options = if preflight.is_some() || signature_verification.is_some() {
        Some(LocalOptions {
            preflight,
            signature_verification,
        })
    } else {
        None
    };
    local_with_opts(local_cmd, api_host, options)
}

pub fn fetch_poll_raw(poll_cmd: &Value, api_host: &str) -> reqwest::blocking::Response {
    let client = create_http_client();
    client.post(&format!("{}/api/v1/poll", api_host))
        .json(poll_cmd)
        .headers(get_headers())
        .send()
        .expect("Failed to send request")
}

pub fn poll(poll_cmd: &Value, api_host: &str) -> Value {
    let res = fetch_poll_raw(poll_cmd, api_host);
    parse_res(res)
}

pub fn fetch_listen_raw(listen_cmd: &Value, api_host: &str) -> reqwest::blocking::Response {
    let client = create_http_client();
    client.post(&format!("{}/api/v1/listen", api_host))
        .json(listen_cmd)
        .headers(get_headers())
        .send()
        .expect("Failed to send request")
}

pub fn listen(listen_cmd: &Value, api_host: &str) -> Value {
    let res = fetch_listen_raw(listen_cmd, api_host);
    parse_res(res)
}

pub fn send_signed(signed_cmd: &Value, api_host: &str) -> Value {
    let client = Client::new();
    let cmd = json!({"cmds": [signed_cmd]});
    let res = client.post(&format!("{}/api/v1/send", api_host))
        .json(&cmd)
        .headers(get_headers())
        .send()
        .expect("Failed to send request");
    parse_res(res)
}
