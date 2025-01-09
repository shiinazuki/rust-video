use anyhow::Result;

use derive_more::{Add, Display, From, Into};

#[derive(PartialEq, Copy, Clone, From, Add, Into, Display, Debug)]
struct MyInt(i32);

#[derive(PartialEq, From)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, From, Add, Display, Debug)]
enum MyEnum {
    #[display("int: {_0}")]
    Int(i32),

    Unit(u32),

    #[display("nothing")]
    Nothing,
}

fn main() -> Result<()> {
    let my_int: MyInt = 10.into();
    let v = my_int + 20.into();
    let v1: i32 = v.into();
    println!("my_int: {}, v: {}, v1: {}", my_int, v, v1);

    let e: MyEnum = 10_i32.into();
    let e1: MyEnum = 20_u32.into();
    let e2 = MyEnum::Nothing;
    println!("e: {}, e1: {}, e2: {}", e, e1, e2);
    println!("e: {:?}, e1: {:?}, e2: {:?}", e, e1, e2);

    Ok(())
}
