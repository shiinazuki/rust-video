mod bundle;

use anyhow::Result;

pub use bundle::run_bundle;

pub type ModulePath = String;
pub type ModuleSource = String;

/// Defines the interface of a module loader.
pub trait ModuleLoader {
    fn load(&self, pecifier: &str) -> Result<ModuleSource>;
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn bundle_ts_should_work() -> Result<()> {
        let ret = run_bundle("fixtures/main.ts", &Default::default())?;
        // assert_eq!(
        // ret,
        // "async function execute(name){console.log(\"Executing lib\");return`Hello ${name}!`;}async function main(){console.log(\"Executing main\");console.log(await execute(\"world\"));}export{main as default};"
        // );
        assert_eq!(
            ret,
            "(function(){async function execute(name){console.log(\"Executing lib\");return`Hello ${name}!`;}async function main(){console.log(\"Executing main\");console.log(await execute(\"world\"));}return{default:main};})();"
        );
        Ok(())
    }
}
