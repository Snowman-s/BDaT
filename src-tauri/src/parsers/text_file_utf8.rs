pub fn parse_text_file_utf8(data: &Vec<u8>) -> serde_json::Value {
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
