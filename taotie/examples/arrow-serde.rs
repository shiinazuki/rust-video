use std::sync::Arc;

use anyhow::Result;
use arrow::{
    array::AsArray as _,
    datatypes::{DataType, Field, Int32Type, Schema},
    json::ReaderBuilder,
};
use serde::Serialize;

fn main() -> Result<()> {
    #[derive(Serialize)]
    struct MyStruct {
        int32: i32,
        string: String,
    }

    let schema = Schema::new(vec![
        Field::new("int32", DataType::Int32, false),
        Field::new("string", DataType::Utf8, false),
    ]);

    let rows = vec![
        MyStruct {
            int32: 5,
            string: "bar".to_string(),
        },
        MyStruct {
            int32: 8,
            string: "foo".to_string(),
        },
    ];

    let mut decoder = ReaderBuilder::new(Arc::new(schema)).build_decoder()?;
    decoder.serialize(&rows)?;
    println!("after={:#?}", decoder);

    let Some(batch) = decoder.flush()? else {
        return Err(anyhow::anyhow!("No batch found"));
    };
    println!("{:#?}", batch);

    let int32 = batch.column(0).as_primitive::<Int32Type>();
    println!("int32={:#?}", int32.values());

    let string = batch.column(1).as_string::<i32>();
    println!("string={} {}", string.value(0), string.value(1));
    Ok(())
}
