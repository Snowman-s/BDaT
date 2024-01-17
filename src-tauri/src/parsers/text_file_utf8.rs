pub fn ask_name(parent_id: &str, _: &serde_json::Value) -> String {
    match parent_id {
        "テキストファイル" => "行".into(),
        _ => "".into(),
    }
}

pub fn ask_explain(id: &str, raw_bytes: &[u8], _: &serde_json::Value) -> String {
    match id {
        "テキストファイル" => "テキストファイル".into(),
        "行" => String::from_utf8_lossy(raw_bytes).to_string(),
        _ => "".into(),
    }
}
