// Utils module: conversions, helpers
pub fn bin_to_int(bin: &str) -> u64 {
    u64::from_str_radix(bin, 2).unwrap_or(0)
}

pub fn int_to_bin(num: u64) -> String {
    format!("{:b}", num)
}

pub fn hex_to_int(hex: &str) -> u64 {
    u64::from_str_radix(hex, 16).unwrap_or(0)
}

pub fn int_to_hex(num: u64) -> String {
    format!("{:x}", num)
}

// ...other helpers to be implemented...
    use serde_json::json;

    pub fn as_list<T: Clone>(item: T) -> Vec<T> {
        vec![item]
    }

    #[derive(Debug, Clone)]
    pub struct KeyPair {
        pub public_key: String,
        pub secret_key: String,
        pub clist: Option<Vec<serde_json::Value>>, // capability objects: {name, args}
    }

    pub fn mk_signer(kp: &KeyPair) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert("pubKey".to_string(), serde_json::Value::String(kp.public_key.clone()));
        if let Some(clist) = &kp.clist {
            obj.insert("clist".to_string(), serde_json::Value::Array(clist.clone()));
        }
        serde_json::Value::Object(obj)
    }

    pub fn pull_sig(s: &serde_json::Value) -> serde_json::Value {
        match s.get("sig") {
            Some(sig) => json!({"sig": sig}),
            None => panic!("Expected to find keys of name 'sig' in {:?}", s),
        }
    }

    pub fn pull_check_hashs(sigs: &[serde_json::Value]) -> String {
        let hsh = sigs[0].get("hash").and_then(|v| v.as_str()).unwrap_or("");
        for sig in sigs {
            let sig_hash = sig.get("hash").and_then(|v| v.as_str()).unwrap_or("");
            if sig_hash != hsh {
                panic!("Sigs for different hashes found: {:?}", sigs);
            }
        }
        hsh.to_string()
    }

    pub fn unique(arr: &[String]) -> Vec<String> {
        use std::collections::HashSet;
        let set: HashSet<_> = arr.iter().cloned().collect();
        set.into_iter().collect()
    }

    pub fn get_headers() -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub fn parse_res(resp: reqwest::blocking::Response) -> serde_json::Value {
        let status = resp.status().as_u16();
        match resp.text() {
            Ok(body) => {
                if let Ok(js) = serde_json::from_str::<serde_json::Value>(&body) {
                    js
                } else {
                    json!({
                        "error": "Failed to parse response as JSON",
                        "status": status,
                        "body": body
                    })
                }
            }
            Err(e) => json!({
                "error": "Failed to read response body",
                "status": status,
                "io_error": e.to_string()
            })
        }
    }
