use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;

#[allow(unused)]
#[derive(Debug, Builder)]
// #[builder(pattern = "owned")]
#[builder(build_fn(name = "priv_build"))]
struct User {
    #[builder(setter(into))]
    name: String,

    #[builder(setter(skip))]
    age: u32,

    #[builder(default = "vec![]", setter(each(name = "skill", into)))]
    skills: Vec<String>,

    #[builder(setter(into, strip_option), default)]
    email: Option<String>,

    #[builder(setter(custom))]
    dob: DateTime<Utc>,
}

impl User {
    fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn dob(&mut self, value: &str) -> &mut Self {
        self.dob = DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();
        self
    }

    pub fn build(&self) -> Result<User> {
        let mut user = self.priv_build()?;
        user.age = (Utc::now().year() - user.dob.year()) as u32;
        Ok(user)
    }
}

fn main() -> Result<()> {
    let user = User::build()
        .name("shiina")
        .skill("game")
        .skill("programming")
        .email("xxx@gmail.com")
        .dob("1999-01-09T16:51:00Z")
        .build()?;
    println!("{:?}", user);

    Ok(())
}
