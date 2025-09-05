// Lang module: meta, cap, exp construction
// ...to be implemented...
use serde_json::json;

pub fn mk_meta(sender: &str, chain_id: &str, gas_price: f64, gas_limit: u64, creation_time: u64, ttl: u64) -> serde_json::Value {
	json!({
		"creationTime": creation_time,
		"ttl": ttl,
		"gasLimit": gas_limit,
		"chainId": chain_id,
		"gasPrice": gas_price,
		"sender": sender
	})
}

pub fn mk_cap(role: &str, description: &str, name: &str, args: Vec<serde_json::Value>) -> serde_json::Value {
	json!({
		"role": role,
		"description": description,
		"cap": {
			"name": name,
			"args": args
		}
	})
}

pub fn mk_exp(module_and_function: &str, namespace: Option<&str>, kwargs: Vec<(&str, serde_json::Value)>) -> String {
	let mut string = String::new();
	if let Some(ns) = namespace {
		string.push_str(&format!("({}.{}", ns, module_and_function));
	} else {
		string.push_str(&format!("({} ", module_and_function));
	}
	for (key, value) in kwargs {
		let val_str = match value {
			serde_json::Value::String(ref s) if s.starts_with('(') || s.starts_with('[') => s.clone(),
			_ => value.to_string(),
		};
		string.push_str(&format!(" {}", val_str));
	}
	string.push(')');
	string
}
