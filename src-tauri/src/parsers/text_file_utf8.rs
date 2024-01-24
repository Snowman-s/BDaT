use super::ParsedData;

pub fn ask_name(parent_id: &str, _: &ParsedData) -> String {
    match parent_id {
        "テキストファイル" => "行".into(),
        _ => "".into(),
    }
}

pub fn ask_explain(id: &str, raw_bytes: &[u8], _: &ParsedData) -> String {
    match id {
        "テキストファイル" => "テキストファイル".into(),
        "行" => String::from_utf8_lossy(raw_bytes).to_string(),
        _ => "".into(),
    }
}
