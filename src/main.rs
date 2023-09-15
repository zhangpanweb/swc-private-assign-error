use std::path::Path;
use swc_common::comments::NoopComments;
use swc_common::errors::HANDLER;
use swc_common::sync::Lrc;
use swc_common::{FileName, Globals, GLOBALS};
use swc_common::{Mark, SourceMap};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax};
use swc_ecma_preset_env::{self, Targets};
use swc_ecma_transforms::feature::FeatureFlag;
use swc_ecma_transforms::helpers::{Helpers, HELPERS};
use swc_ecma_transforms::Assumptions;
use swc_ecma_visit::Fold;
use swc_error_reporters::handler::try_with_handler;

fn main() {
    let source = r#"
    class A {
        b() {
            this.#a ||= 1;
        }
    }
"#;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Real(Path::new("").to_owned()), source.to_owned());

    let lexer = Lexer::new(
        Syntax::Es(EsConfig {
            ..Default::default()
        }),
        swc_ecma_ast::EsVersion::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    let ast = parser.parse_module().unwrap();

    let globals = Globals::default();
    let targets: Targets = serde_json::from_str(r#"{"chrome":74}"#).unwrap();
    GLOBALS.set(&globals, || {
        let _ = try_with_handler(cm.clone(), Default::default(), |handler| {
            HELPERS.set(&Helpers::new(true), || {
                HANDLER.set(handler, || {
                    let mut preset_env = swc_ecma_preset_env::preset_env(
                        Mark::new(),
                        Some(NoopComments),
                        swc_ecma_preset_env::Config {
                            targets: Some(targets),
                            mode: Some(swc_ecma_preset_env::Mode::Entry),
                            ..Default::default()
                        },
                        Assumptions::default(),
                        &mut FeatureFlag::default(),
                    );
                    preset_env.fold_module(ast.clone());
                });
            });

            Ok(())
        });
    });
}
