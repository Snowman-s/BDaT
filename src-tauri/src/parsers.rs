mod int;
mod text_file_utf8;

use std::collections::HashMap;

use ::phf::{phf_map, Map};
use serde::Serialize;

use self::int::calc;

#[derive(Clone, Copy)]
pub enum Parsers {
    TextFileUTF8,
    WindowsBitMap,
}

impl Parsers {
    fn get_syntax(&self) -> &'static [u8] {
        match self {
            Parsers::TextFileUTF8 => include_bytes!("./parsers/text_file_utf8.yaml"),
            Parsers::WindowsBitMap => include_bytes!("./parsers/windows_bitmap.yaml"),
        }
    }

    fn ask_name(&self, parent_id: &str, child: &ParsedData) -> String {
        match self {
            Parsers::TextFileUTF8 => text_file_utf8::ask_name(parent_id, child),
            Parsers::WindowsBitMap => "".into(),
        }
    }

    fn ask_explain(&self, id: &str, raw_bytes: &[u8], data: &ParsedData) -> String {
        match self {
            Parsers::TextFileUTF8 => text_file_utf8::ask_explain(id, raw_bytes, &data),
            Parsers::WindowsBitMap => id.to_string(),
        }
    }
}

pub static PARSERS_LIST: Map<&'static str, Parsers> = phf_map! {
  "Text File(UTF-8)" => Parsers::TextFileUTF8,
  "Windows BitMap" => Parsers::WindowsBitMap,
};

pub fn inner_parse(parser_str: &str, data: Vec<u8>) -> serde_json::Value {
    let parser_opt = PARSERS_LIST.get(parser_str);

    if let None = parser_opt {
        return serde_json::json!(parse_failure(&data, 0).body);
    }

    let parser = parser_opt.unwrap();

    let mut context = ParseContext {
        variables: HashMap::new(),
    };
    let parsed_opt = parse(parser, &data, &mut context).body;

    if let Err(()) = parsed_opt {
        return serde_json::json!(parse_failure(&data, 0).body);
    }

    let mut parsed = parsed_opt.unwrap();

    name(parser, &data, &mut parsed);

    // 既にnamedになった
    serde_json::json!(parsed)
}

// 妥協案 (本当はスタックを使いたかった)
fn name(parser: &Parsers, data: &Vec<u8>, p: &mut ParsedData) {
    for child in &mut p.children {
        name(parser, data, &mut child.data);
        if let Some(id) = &p.id {
            child.name = parser.ask_name(&id, &child.data);
        }
    }
    if let Some(id) = &p.id {
        p.explain = parser.ask_explain(id, &data[p.minIndex..p.maxIndex], p)
    }
}

#[derive(Serialize, Clone, Debug)]
struct ParsedData {
    id: Option<String>,
    explain: String,
    minIndex: usize,
    maxIndex: usize,
    children: Vec<Box<ParsedDataChild>>,
}

#[derive(Serialize, Clone, Debug)]
struct ParsedDataChild {
    data: ParsedData,
    name: String,
}

struct ParseResult {
    body: Result<ParsedData, ()>,
    new_min_index: usize,
}

struct ParseContext {
    variables: HashMap<String, i64>,
}

impl ParseContext {
    fn insert_variable(&mut self, key: String, data: i64) {
        self.variables.insert(key, data);
    }

    fn get_variable(&mut self, key: String) -> Option<&i64> {
        self.variables.get(&key)
    }
}

fn parse_failure(data: &Vec<u8>, min_index: usize) -> ParseResult {
    ParseResult {
        body: Ok(ParsedData {
            id: None,
            explain: "解析失敗".to_string(),
            minIndex: min_index,
            maxIndex: data.len(),
            children: vec![],
        }),
        new_min_index: data.len(),
    }
}

fn parse(parser: &Parsers, data: &Vec<u8>, context: &mut ParseContext) -> ParseResult {
    let binding = serde_yaml::from_slice::<serde_yaml::Value>(parser.get_syntax()).unwrap();
    let def = binding.as_mapping().unwrap();

    parse_main(def, data, 0, context)
}

fn parse_main(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let behavior = def.get("type").and_then(|t| t.as_str());

    match behavior {
        Some("repeat") => parse_repeat(def, data, min_index, context),
        Some("repeat0") => parse_repeat0(def, data, min_index, context),
        Some("until_byte") => parse_until_byte(def, data, min_index, context),
        Some("tuple") => parse_tuple(def, data, min_index, context),
        Some("constants") => parse_constants(def, data, min_index, context),
        Some("u16") => parse_u16(def, data, min_index, context),
        Some("u32") => parse_u32(def, data, min_index, context),
        Some("skim") => parse_skim(def, data, min_index, context),
        _ => parse_failure(data, min_index),
    }
}

fn parse_repeat(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let repeat_times_def = def.get("repeat_times").unwrap();
    let child_def = def.get("children").unwrap().as_mapping().unwrap();

    let repeat_times = calc(repeat_times_def, context);
    if repeat_times.is_none() {
        return parse_failure(data, min_index);
    }
    let repeat_times = repeat_times.unwrap();

    let mut children: Vec<ParsedData> = vec![];

    let mut mut_min_index = min_index;

    for _ in 0..repeat_times {
        if data.len() < mut_min_index {
            break;
        }

        let result = parse_main(child_def, data, mut_min_index, context);
        if let Ok(value) = result.body {
            children.push(value);
            mut_min_index = result.new_min_index;
        } else {
            break;
        }
    }

    let mut children_with_name = vec![];
    for child in children {
        children_with_name.push(Box::from(ParsedDataChild {
            name: "".into(),
            data: child,
        }));
    }
    let body = ParsedData {
        minIndex: min_index,
        maxIndex: mut_min_index,
        children: children_with_name,
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: mut_min_index,
    }
}

fn parse_repeat0(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let child_def = def.get("children").unwrap().as_mapping().unwrap();

    let mut children: Vec<ParsedData> = vec![];

    let mut mut_min_index = min_index;

    loop {
        if data.len() <= mut_min_index {
            break;
        }

        let result = parse_main(child_def, data, mut_min_index, context);
        if let Ok(value) = result.body {
            children.push(value);
            mut_min_index = result.new_min_index;
        } else {
            break;
        }
    }

    let mut children_with_name = vec![];
    for child in children {
        children_with_name.push(Box::from(ParsedDataChild {
            name: "".into(),
            data: child,
        }));
    }
    let body = ParsedData {
        minIndex: min_index,
        maxIndex: mut_min_index,
        children: children_with_name,
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: mut_min_index,
    }
}

fn parse_until_byte(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let byte_def: u8 = def
        .get("byte")
        .and_then(|v| v.as_u64())
        .unwrap()
        .try_into()
        .unwrap();

    let mut new_min_index = data.len();
    for i in min_index..data.len() {
        if data[i] == byte_def {
            new_min_index = i + 1;
            break;
        }
    }

    let body = ParsedData {
        minIndex: min_index,
        maxIndex: new_min_index,
        children: vec![],
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index,
    }
}

fn parse_tuple(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let items_def = def.get("items").unwrap().as_sequence().unwrap();

    let mut children: Vec<Box<ParsedDataChild>> = vec![];

    let mut mut_min_index = min_index;

    for items in items_def {
        if data.len() < mut_min_index {
            children.push(Box::new(ParsedDataChild {
                name: "*".into(),
                data: parse_failure(data, mut_min_index).body.unwrap(),
            }));
            break;
        }

        let result = parse_main(items.as_mapping().unwrap(), data, mut_min_index, context);
        if let Ok(value) = result.body {
            children.push(Box::new(ParsedDataChild {
                data: value,
                name: "".into(),
            }));
            mut_min_index = result.new_min_index;
        } else {
            break;
        }
    }

    let body = ParsedData {
        minIndex: min_index,
        maxIndex: mut_min_index,
        children,
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: mut_min_index,
    }
}

fn parse_constants(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let constants_def = def.get("constants").unwrap().as_sequence().unwrap();

    let mut mut_min_index = min_index;

    for v in constants_def {
        let n = v.as_u64().unwrap();
        if data.len() <= mut_min_index || data[mut_min_index] != u8::try_from(n).unwrap() {
            return parse_failure(data, min_index);
        }
        mut_min_index += 1;
    }

    let body = ParsedData {
        minIndex: min_index,
        maxIndex: mut_min_index,
        children: vec![],
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: mut_min_index,
    }
}

fn parse_u16(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());

    if data.len() < min_index + 2 {
        return parse_failure(data, min_index);
    }

    if let Some(id) = id_def {
        context.insert_variable(
            id.to_owned(),
            u16::from_le_bytes(data[min_index..min_index + 2].try_into().unwrap()).into(),
        );
    }

    let body = ParsedData {
        minIndex: min_index,
        maxIndex: min_index + 2,
        children: vec![],
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: min_index + 2,
    }
}

fn parse_u32(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());

    if data.len() < min_index + 4 {
        return parse_failure(data, min_index);
    }

    if let Some(id) = id_def {
        context.insert_variable(
            id.to_owned(),
            u32::from_le_bytes(data[min_index..min_index + 4].try_into().unwrap()).into(),
        );
    }

    let body = ParsedData {
        minIndex: min_index,
        maxIndex: min_index + 4,
        children: vec![],
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: min_index + 4,
    }
}

fn parse_skim(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    context: &mut ParseContext,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let len_bytes_def = def.get("len_bytes").unwrap();

    let len_bytes: Option<usize> =
        int::calc(len_bytes_def, context).and_then(|l| l.try_into().ok());
    if len_bytes.is_none() {
        return parse_failure(data, min_index);
    }
    let len_bytes: usize = len_bytes.unwrap();

    if data.len() < min_index + len_bytes {
        return parse_failure(data, min_index);
    }

    let body = ParsedData {
        minIndex: min_index,
        maxIndex: min_index + len_bytes,
        children: vec![],
        id: id_def.map(|s| s.to_string()),
        explain: "".into(),
    };

    ParseResult {
        body: Ok(body),
        new_min_index: min_index + len_bytes,
    }
}
