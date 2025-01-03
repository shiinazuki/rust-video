use crate::cli::OutputFormat;
use csv::ReaderBuilder;
use serde_json::Value;
use std::fs;

pub fn process_csv(
    input: &str,
    output: String,
    delimiter: char,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .from_path(input)?;

    // 依赖具体类型
    // let records = reader
    //     .deserialize()
    //     .map(|record| record.unwrap())
    //     .collect::<Vec<Record>>();

    // 不依赖具体类型
    let headers = reader.headers()?.clone();
    let records = reader
        .records()
        .map(|record| {
            let record = record.unwrap();
            println!("{:?}", record);
            headers.iter().zip(record.iter()).collect::<Value>()
        })
        .collect::<Vec<Value>>();

    // println!("{:#?}", records);

    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&records)?,
        OutputFormat::Yaml => serde_yaml::to_string(&records)?,
    };

    // println!("{}", content);
    fs::write(output, content)?;

    Ok(())
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Record {
//     #[serde(rename = "用户名")]
//     pub username: String,
//     #[serde(rename = "密码")]
//     pub password: String,
//     #[serde(rename = "年龄")]
//     pub age: String,
//     #[serde(rename = "时间")]
//     pub create_time: String,
// }
