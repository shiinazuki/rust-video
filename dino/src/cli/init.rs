use std::{fs, path::Path};

use anyhow::Result;
use askama::Template;
use clap::Parser;
use dialoguer::Input;
use git2::Repository;

use crate::CmdExector;

#[derive(Debug, Parser)]
pub struct InitOpts {}

#[derive(Template)]
#[template(path = "config.yaml.j2")]
struct ConfigFile {
    name: String,
}

#[derive(Template)]
#[template(path = ".gitignore.j2")]
struct GitIgnoreFile {}

#[derive(Template)]
#[template(path = "main.ts.j2")]
struct MainTsFile {}

impl CmdExector for InitOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let name: String = Input::new().with_prompt("Project name").interact_text()?;

        let cur = Path::new(".");
        if fs::read_dir(".")?.next().is_none() {
            init_project(&name, Path::new(cur))?;
        } else {
            let path = cur.join(&name);
            init_project(&name, &path)?;
        }

        Ok(())
    }
}

fn init_project(name: &str, path: &Path) -> Result<()> {
    Repository::init(path)?;
    let config = ConfigFile {
        name: name.to_string(),
    };
    fs::write(path.join("config.yaml"), config.render()?)?;
    fs::write(path.join("main.ts"), MainTsFile {}.render()?)?;
    fs::write(path.join(".gitignore"), GitIgnoreFile {}.render()?)?;

    Ok(())
}
