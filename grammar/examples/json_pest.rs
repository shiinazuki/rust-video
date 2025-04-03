use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../examples/json_pest.pest"]
struct JsonParser;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn main() -> Result<()> {
    let path = env::current_dir()?
        .join("grammar")
        .join("assets")
        .join("json_log.txt");
    let file = File::open(path)?;
    let buf_read = BufReader::new(file);

    let mut pest_vec = Vec::new();

    for line in buf_read.lines() {
        let line = &line?;
        let parsed = JsonParser::parse(Rule::json, &line)?
            .next()
            .ok_or_else(|| anyhow::anyhow!("json has no value"))?;

        let value = parse_value(parsed)?;

        pest_vec.push(value);
    }
    print_info(&pest_vec[0]);
    Ok(())
}

fn print_info(json_value: &JsonValue) {
    if let JsonValue::Object(obj) = json_value {
        if let Some(JsonValue::String(name)) = obj.get("name") {
            println!("姓名: {}", name);
        }
        if let Some(JsonValue::Number(age)) = obj.get("age") {
            println!("年龄: {}", age);
        }
        if let Some(JsonValue::Bool(is_student)) = obj.get("is_student") {
            println!("是学生吗: {}", is_student);
        }
        if let Some(JsonValue::Array(marks)) = obj.get("marks") {
            let labels = ["经度", "纬度", "海拔"];
            for (i, mark_value) in marks.iter().enumerate() {
                if i < labels.len() {
                    if let Some(label) = labels.get(i) {
                        if let JsonValue::Number(value) = mark_value {
                            println!("{}: {}", label, value);
                        }
                    }
                }
            }
        }
        if let Some(JsonValue::Object(address_obj)) = obj.get("address") {
            if let Some(JsonValue::String(city)) = address_obj.get("city") {
                println!("城市: {}", city);
            }
            if let Some(JsonValue::Number(zip)) = address_obj.get("zip") {
                println!("邮编: {}", zip);
            }
        }
    }
}

fn parse_array(pair: Pair<Rule>) -> Result<Vec<JsonValue>> {
    pair.into_inner().map(parse_value).collect()
}

fn parse_object(pair: Pair<Rule>) -> Result<HashMap<String, JsonValue>> {
    let inner = pair.into_inner();
    let values = inner.map(|pair| {
        let mut inner = pair.into_inner();
        let key = inner
            .next()
            .map(|p| p.as_str().to_string())
            .ok_or_else(|| anyhow::anyhow!("expected key in object, found none"))?;
        let value = parse_value(
            inner
                .next()
                .ok_or_else(|| anyhow::anyhow!("expected value in object, found none"))?,
        )?;
        Ok((key, value))
    });
    values.collect::<Result<HashMap<_, _>>>()
}

fn parse_value(pair: Pair<Rule>) -> Result<JsonValue> {
    let ret = match pair.as_rule() {
        Rule::null => JsonValue::Null,
        Rule::bool => JsonValue::Bool(pair.as_str().parse()?),
        Rule::number => JsonValue::Number(pair.as_str().parse()?),
        Rule::chars => JsonValue::String(pair.as_str().to_string()),
        Rule::array => JsonValue::Array(parse_array(pair)?),
        Rule::object => JsonValue::Object(parse_object(pair)?),
        Rule::value => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow::anyhow!("expected value"))?;
            parse_value(inner)?
        }
        v => {
            panic!("unhandled rule: {:?}", v);
        }
    };
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use pest::consumes_to;
    use pest::parses_to;

    use super::*;

    #[test]
    fn pest_parse_null_should_work() -> Result<()> {
        let input = "null";
        let parsed = JsonParser::parse(Rule::null, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Null, result);
        Ok(())
    }

    #[test]
    fn pest_parse_bool_should_work() -> Result<()> {
        let input = "true";
        let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Bool(true), result);

        let input = "false";
        let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Bool(false), result);

        Ok(())
    }

    #[test]
    fn pest_parse_number_should_work() -> Result<()> {
        let input = "123";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(123.0), result);

        let input = "-123";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(-123.0), result);

        let input = "123.456";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(123.456), result);

        let input = "-123.456";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(-123.456), result);

        Ok(())
    }

    #[test]
    fn pest_parse_string_should_work() -> Result<()> {
        let input = r#""hello \"world\"""#;
        let parsed = JsonParser::parse(Rule::string, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::String(r#"hello \"world\""#.to_string()), result);

        Ok(())
    }

    #[test]
    fn pest_parse_array_should_work() -> Result<()> {
        let input = r#"[1, 2, 3]"#;
        let parsed = JsonParser::parse(Rule::array, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(
            JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0)
            ]),
            result
        );

        Ok(())
    }

    #[test]
    fn pest_parse_object_should_work() -> Result<()> {
        let input = r#"{"a": 1, "b": 2, "c": 3}"#;
        let parsed = JsonParser::parse(Rule::object, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(
            JsonValue::Object(
                vec![
                    ("a".to_string(), JsonValue::Number(1.0)),
                    ("b".to_string(), JsonValue::Number(2.0)),
                    ("c".to_string(), JsonValue::Number(3.0))
                ]
                .into_iter()
                .collect()
            ),
            result
        );

        Ok(())
    }

    #[test]
    fn pest_parse_rule_should_work() {
        parses_to! {
            parser: JsonParser,
            input: r#"{ "hello": "world" }"#,
            rule: Rule::json,
            tokens: [
                object(0, 20, [
                    pair(2, 18, [
                        chars(3, 8),
                        value(11, 18, [
                            chars(12, 17)
                        ])
                    ])
                ])
            ]
        }
    }
}
