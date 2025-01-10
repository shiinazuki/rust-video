use anyhow::Result;
use strum::{
    EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr,
    VariantNames,
};

#[allow(unused)]
#[derive(
    Debug, EnumString, EnumCount, EnumDiscriminants, EnumIter, EnumIs, VariantNames, IntoStaticStr,
)]
enum MyEnum {
    A,
    B(String),
    C,
}

fn main() -> Result<()> {
    println!("{:?}", MyEnum::VARIANTS);
    MyEnum::iter().for_each(|v| println!("{:?}", v));

    let my_enum = MyEnum::B("hello".to_string());
    println!("{:?}", my_enum.is_b());
    let s: &'static str = my_enum.into();
    println!("{}", s);
    Ok(())
}
