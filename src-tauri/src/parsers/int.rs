use super::ParseContext;

pub fn calc(def: &serde_yaml::Value, context: &mut ParseContext) -> Option<i64> {
    if let Some(int) = def.as_i64() {
        Some(int)
    } else if let Some(string) = def.as_str() {
        context.get_variable(string.to_string()).map(|i| i.clone())
    } else if let Some(mapping) = def.as_mapping() {
        if let Some(array) = mapping.get("+").and_then(|d| d.as_sequence()) {
            let mut summing = 0;
            for v in array {
                let calced = calc(v, context);
                match calced {
                    Some(c) => summing += c,
                    None => return None,
                }
            }
            Some(summing)
        } else if let Some(array) = mapping.get("*").and_then(|d| d.as_sequence()) {
            let mut mul = 1;
            for v in array {
                let calced = calc(v, context);
                match calced {
                    Some(c) => mul *= c,
                    None => return None,
                }
            }
            Some(mul)
        } else if let Some(operator) = mapping.get("//") {
            let input = mapping.get("input").unwrap();

            calc(input, context).and_then(|inp| calc(operator, context).map(|ope| (inp / ope)))
        } else if let Some(operator) = mapping.get("to_floor_multiple_of") {
            let input = mapping.get("input").unwrap();

            calc(input, context)
                .and_then(|inp| calc(operator, context).map(|ope| ((inp + ope - 1) / ope * ope)))
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}
