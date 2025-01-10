use anyhow::Result;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Builder, Serialize, Deserialize)]
#[builder(pattern = "owned")]
struct User {
    #[builder(setter(into))]
    name: String,

    #[builder(setter(into))]
    age: u8,

    #[builder(setter(each(name = "skill", into)))]
    skills: Vec<String>,
}

impl User {
    fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

fn main() -> Result<()> {
    let user = User::build()
        .name("shiina")
        .age(12)
        .skill("game")
        .skill("yellow")
        .build()?;

    let user_json = serde_json::to_string(&user)?;
    println!("user_json = {}", user_json);
    
    let user_str: User = serde_json::from_str(&user_json)?;
    println!("{:#?}", user);
    
    assert_eq!(user, user_str);
    Ok(())
}
