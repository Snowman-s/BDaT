use std::default;

pub fn ask_name(parent_id: &str, child: &serde_json::Value) -> Option<String> {
    match parent_id {
        "テキストファイル" => Some("行".into()),
        default => None,
    }
}

pub fn ask_explain(id: &str, raw_bytes: &[u8], data: &serde_json::Value) -> Option<String> {
    match id {
        "テキストファイル" => Some("テキストファイル".into()),
        "行" => Some(String::from_utf8_lossy(raw_bytes).to_string()),
        default => None,
    }
}
