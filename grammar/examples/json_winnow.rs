use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use winnow::{
    Parser,
    ascii::{digit1, multispace0},
    combinator::{alt, delimited, opt, separated, separated_pair, trace},
    error::{ContextError, ErrMode, ParserError},
    stream::{AsChar, Stream, StreamIsPartial},
    token::take_until,
};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Num {
    Int(i64),
    Float(f64),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(Num),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[allow(unused)]
#[derive(Debug)]
struct Person {
    name: String,
    age: u8,
    is_statudent: bool,
    marsk: Vec<f64>,
    address: Address,
}

#[allow(unused)]
#[derive(Debug)]
struct Address {
    city: String,
    zip: u32,
}

fn main() -> Result<()> {
    let path = env::current_dir()?
        .join("grammar")
        .join("assets")
        .join("json_log.txt");

    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    let mut json_log = Vec::new();

    for line in buf_reader.lines() {
        let line = line?;

        let v = parse_json(&line).map_err(|e| anyhow::anyhow!("Failed to parse JSON: {:?}", e))?;

        json_log.push(v);
    }
    println!("{:#?}", json_log[0]);
    print_info(&json_log[0]);

    Ok(())
}

fn print_info(json_value: &JsonValue) {
    if let JsonValue::Object(obj) = json_value {
        if let Some(JsonValue::String(name)) = obj.get("name") {
            println!("姓名: {}", name);
        }
        if let Some(JsonValue::Number(Num::Int(age))) = obj.get("age") {
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
                        if let JsonValue::Number(Num::Float(value)) = mark_value {
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
            if let Some(JsonValue::Number(Num::Int(zip))) = address_obj.get("zip") {
                println!("邮编: {}", zip);
            }
        }
    }
}

fn parse_json(input: &str) -> Result<JsonValue> {
    let input = &mut (&*input);
    parse_value(input).map_err(|e| anyhow::anyhow!("Failed to parse Json: {:?}", e))
}

fn sep_with_space<Input, Output, Error, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<Input, (), Error>
where
    Input: Stream + StreamIsPartial,
    <Input as Stream>::Token: AsChar + Clone,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
{
    trace("sep_with_space", move |input: &mut Input| {
        let _ = multispace0.parse_next(input)?;
        parser.parse_next(input)?;
        multispace0.parse_next(input)?;
        Ok(())
    })
}

fn parse_null(input: &mut &str) -> winnow::Result<()> {
    "null".value(()).parse_next(input)
}

fn parse_bool(input: &mut &str) -> winnow::Result<bool> {
    alt(("true", "false")).parse_to().parse_next(input)
}

fn parse_num(input: &mut &str) -> winnow::Result<Num> {
    let sign = opt("-").map(|s| s.is_some()).parse_next(input)?;
    let num = digit1.parse_to::<i64>().parse_next(input)?;
    let ret: Result<(), ErrMode<ContextError>> = ".".value(()).parse_next(input);
    if ret.is_ok() {
        let frac = digit1.parse_to::<i64>().parse_next(input)?;
        let v = format!("{}.{}", num, frac).parse::<f64>().unwrap();
        Ok(if sign {
            Num::Float(-v as _)
        } else {
            Num::Float(v as _)
        })
    } else {
        Ok(if sign { Num::Int(-num) } else { Num::Int(num) })
    }
}

fn parse_string(input: &mut &str) -> winnow::Result<String> {
    let ret = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(ret.to_string())
}

fn parse_object(input: &mut &str) -> winnow::Result<HashMap<String, JsonValue>> {
    let sep1 = sep_with_space('{');
    let sep2 = sep_with_space('}');
    let sep_comma = sep_with_space(',');
    let sep_colon = sep_with_space(':');

    let parse_kv_pair = separated_pair(parse_string, sep_colon, parse_value);
    let parse_kv = separated(1.., parse_kv_pair, sep_comma);
    delimited(sep1, parse_kv, sep2).parse_next(input)
}
fn parse_array(input: &mut &str) -> winnow::Result<Vec<JsonValue>> {
    let sep1 = sep_with_space('[');
    let sep2 = sep_with_space(']');
    let sep_comma = sep_with_space(',');
    let parse_values = separated(0.., parse_value, sep_comma);
    delimited(sep1, parse_values, sep2).parse_next(input)
}

fn parse_value(input: &mut &str) -> winnow::Result<JsonValue> {
    alt((
        parse_null.value(JsonValue::Null),
        parse_bool.map(JsonValue::Bool),
        parse_num.map(JsonValue::Number),
        parse_string.map(JsonValue::String),
        parse_array.map(JsonValue::Array),
        parse_object.map(JsonValue::Object),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() -> winnow::Result<()> {
        let input = "null";
        let ret = parse_null(&mut (&*input))?;
        assert_eq!(ret, ());
        Ok(())
    }

    #[test]
    fn test_parse_bool() -> winnow::Result<()> {
        let input = "true";
        let ret = parse_bool(&mut (&*input))?;
        assert!(ret);

        let input = "false";
        let ret = parse_bool(&mut (&*input))?;
        assert!(!ret);

        Ok(())
    }

    #[test]
    fn test_parse_num() -> winnow::Result<()> {
        let input = "123";
        let ret = parse_num(&mut (&*input))?;
        assert_eq!(ret, Num::Int(123));

        let input = "-123";
        let ret = parse_num(&mut (&*input))?;
        assert_eq!(ret, Num::Int(-123));

        let input = "123.456";
        let ret = parse_num(&mut (&*input))?;
        assert_eq!(ret, Num::Float(123.456));

        let input = "-123.456";
        let ret = parse_num(&mut (&*input))?;
        assert_eq!(ret, Num::Float(-123.456));
        Ok(())
    }

    #[test]
    fn test_parse_string() -> winnow::Result<()> {
        let input = r#""hello""#;
        let ret = parse_string(&mut (&*input))?;
        assert_eq!(ret, "hello");
        Ok(())
    }

    #[test]
    fn test_parse_array() -> winnow::Result<()> {
        let input = r#"[1, 2, 3]"#;
        let ret = parse_array(&mut (&*input))?;
        assert_eq!(
            ret,
            vec![
                JsonValue::Number(Num::Int(1)),
                JsonValue::Number(Num::Int(2)),
                JsonValue::Number(Num::Int(3)),
            ]
        );

        let input = r#"["a", "b", "c"]"#;
        let ret = parse_array(&mut (&*input))?;
        assert_eq!(
            ret,
            vec![
                JsonValue::String("a".to_string()),
                JsonValue::String("b".to_string()),
                JsonValue::String("c".to_string()),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_object() -> winnow::Result<()> {
        let input = r#"{"a": 1, "b": 2}"#;
        let ret = parse_object(&mut (&*input))?;
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), JsonValue::Number(Num::Int(1)));
        expected.insert("b".to_string(), JsonValue::Number(Num::Int(2)));
        assert_eq!(ret, expected);

        let input = r#"{"a": 1, "b": [1, 2, 3]}"#;
        let ret = parse_object(&mut (&*input))?;
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), JsonValue::Number(Num::Int(1)));
        expected.insert(
            "b".to_string(),
            JsonValue::Array(vec![
                JsonValue::Number(Num::Int(1)),
                JsonValue::Number(Num::Int(2)),
                JsonValue::Number(Num::Int(3)),
            ]),
        );
        assert_eq!(ret, expected);

        Ok(())
    }
}
