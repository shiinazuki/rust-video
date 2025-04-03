use anyhow::Result;
use rand::Rng;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Address {
    city: String,
    zip: u32,
}

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    is_student: bool,
    marks: Vec<f64>,
    address: Address,
}

fn generate_random_data(num_records: u32) -> Result<()> {
    let file_path = "grammar/assets/json_log.txt";
    let mut file = File::options().append(true).create(true).open(file_path)?;

    let mut rng = rand::rng();
    let first_names = ["张", "王", "李", "赵", "钱", "孙", "周", "吴", "郑", "冯"];
    let last_names = ["伟", "芳", "娜", "明", "杰", "静", "丽", "强", "军", "涛"];
    let cities = [
        "北京", "上海", "广州", "深圳", "杭州", "南京", "成都", "重庆", "武汉", "西安",
    ];

    for _ in 0..num_records {
        let name = format!(
            "{}{}",
            first_names.choose(&mut rng).unwrap(),
            last_names.choose(&mut rng).unwrap()
        );
        let age = rng.random_range(18..=65);
        let is_student = rng.random_bool(0.5);
        let marks = (0..3)
            .map(|_| rng.random_range(-100.0..=100.0))
            .collect::<Vec<f64>>();
        let address = Address {
            city: cities.choose(&mut rng).unwrap().to_string(),
            zip: rng.random_range(10000..=99999),
        };
        let person = Person {
            name,
            age,
            is_student,
            marks,
            address,
        };
        let json_string = serde_json::to_string(&person)?;
        writeln!(file, "{}", json_string)?;
    }
    Ok(())
}

fn main() {
    let num_records = 50000;
    generate_random_data(num_records).unwrap();

    println!(
        "成功生成 {} 条随机数据，并保存到 random_data.json 文件中。",
        num_records
    );
}
