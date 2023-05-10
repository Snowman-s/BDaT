use ::phf::{phf_map, Map};

#[derive(Clone, Copy)]
pub enum Parsers {
    TextFileUTF8,
}

pub static PARSERS_LIST: Map<&'static str, Parsers> = phf_map! {
  "Text File(UTF-8)" => Parsers::TextFileUTF8,
};

pub fn inner_parse(parser: &str, data: Vec<u8>) -> serde_json::Value {
    PARSERS_LIST
        .get(parser)
        .cloned()
        .map_or_else(
            || parse_failure(&data),
            |parser| match parser {
                Parsers::TextFileUTF8 => parse_text_file_utf8(&data),
            },
        )
        .into()
}

fn parse_failure(data: &Vec<u8>) -> serde_json::Value {
    serde_json::json!({
      "explain": "解析失敗",
      "range": {
        "minIndex": 0,
        "mqxIndex": data.len()
      },
      "children":[]
    })
}

fn parse_text_file_utf8(data: &Vec<u8>) -> serde_json::Value {
    let mut children: serde_json::Value = serde_json::json!([]);

    let mut now_vec: Vec<u8> = vec![];
    let mut min_index = 0;
    let mut max_index = -1;
    for d in data {
        now_vec.append(&mut vec![d.clone()]);
        max_index += 1;
        if d == &b'\n' {
            let explain = String::from_utf8(now_vec).unwrap_or("???".into());
            children.as_array_mut().unwrap().append(&mut vec![
                serde_json::json!({
                  "name": "行",
                  "data": { "explain": explain, "minIndex": min_index, "maxIndex": max_index, "children":[] }
                }),
            ]);
            now_vec = vec![];
            min_index = max_index + 1;
        }
    }

    if now_vec.len() != 0 {
        let explain = String::from_utf8(now_vec).unwrap_or("???".into());
        children.as_array_mut().unwrap().append(&mut vec![
          serde_json::json!({
            "name": "行",
            "data": { "explain": explain, "minIndex": min_index, "maxIndex": max_index, "children":[] }
          }),
      ]);
    }

    serde_json::json!({
      "explain": "テキストファイル",
      "minIndex": 0,
      "maxIndex": data.len(),
      "children": children
    })
}
