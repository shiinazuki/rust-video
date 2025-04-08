use anyhow::{Result, bail};
use base64::{Engine, prelude::BASE64_STANDARD};
use swc_common::{
    BytePos, FileName, FilePathMapping, GLOBALS, Globals, LineCol, Mark, SourceMap,
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
};
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;

pub struct TypeScript;

impl TypeScript {
    /// Compiles TypeScript code into JavaScript.
    pub fn compile(filename: Option<&str>, source: &str) -> Result<String> {
        let globals = Globals::default();
        let cm: Lrc<SourceMap> = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
        let comments = SingleThreadedComments::default();

        let filename = match filename {
            Some(filename) => FileName::Custom(filename.into()),
            None => FileName::Anon,
        };

        let fm = cm.new_source_file(filename.into(), source.into());

        // Initialize the TypeScript lexer.
        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                no_early_errors: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        let program = match parser
            .parse_program()
            .map_err(|e| e.into_diagnostic(&handler).emit())
        {
            Ok(module) => module,
            Err(_) => bail!("TypeScript compilation failed."),
        };

        // This is where we're gonna store the JavaScript output.
        let mut output = vec![];
        let mut source_map = vec![];

        GLOBALS.set(&globals, || {
            // We're gonna apply the following transformations.
            //
            // 1. Conduct identifier scope analysis.
            // 2. Remove typescript types.
            // 3. Fix up any identifiers with the same name, but different contexts.
            // 4. Ensure that we have enough parenthesis.
            //
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            let program = program
                .apply(resolver(unresolved_mark, top_level_mark, true))
                .apply(strip(unresolved_mark, top_level_mark))
                .apply(hygiene())
                .apply(fixer(Some(&comments)));

            {
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config::default(),
                    cm: cm.clone(),
                    comments: None,
                    wr: JsWriter::new(cm.clone(), "\n", &mut output, Some(&mut source_map)),
                };

                emitter.emit_program(&program).unwrap();
            }
        });

        // Prepare the inline source map comment.
        let source_map = source_map_to_string(cm, &source_map);
        let source_map = BASE64_STANDARD.encode(source_map.as_bytes());
        let source_map = format!(
            "//# sourceMappingURL=data:application/json;base64,{}",
            source_map
        );

        let code = String::from_utf8_lossy(&output).to_string();
        let output = format!("{}\n{}", code, source_map);

        Ok(output)
    }
}

/// Returns the string (JSON) representation of the source-map.
fn source_map_to_string(cm: Lrc<SourceMap>, mappings: &[(BytePos, LineCol)]) -> String {
    let mut buffer = Vec::new();
    let source_map = cm.build_source_map(mappings);
    source_map.to_writer(&mut buffer).unwrap();
    String::from_utf8_lossy(&buffer).to_string()
}
