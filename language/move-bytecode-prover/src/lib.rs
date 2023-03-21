// pub mod converter;
pub mod expansion_from_source_map;
// #![forbid(unsafe_code)]

// use crate::cli::Options;
// use anyhow::anyhow;
// use codespan_reporting::{
//     diagnostic::Severity,
//     term::termcolor::{Buffer, ColorChoice, StandardStream, WriteColor},
// };
// #[allow(unused_imports)]
// use log::{debug, info, warn};
// use move_abigen::Abigen;
// use move_compiler::shared::PackagePaths;
// use move_docgen::Docgen;
// use move_errmapgen::ErrmapGen;
// use move_model::{
//     code_writer::CodeWriter, model::GlobalEnv, parse_addresses_from_options,
//     run_model_builder_with_options,
// };
// use move_prover_boogie_backend::{
//     add_prelude, boogie_wrapper::BoogieWrapper, bytecode_translator::BoogieTranslator,
// };
// use move_stackless_bytecode::{
//     escape_analysis::EscapeAnalysisProcessor,
//     function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
//     pipeline_factory,
//     read_write_set_analysis::{self, ReadWriteSetProcessor},
// };
// use std::{
//     collections::BTreeSet,
//     fs,
//     path::{Path, PathBuf},
//     time::Instant,
// };

// pub mod cli;

// use crate bytecode_model_builder_with_specs.rs()

// pub fn run_bytecode_prover<W: WriteColor>(
//     error_writer: &mut W,
//     options: Options,
// ) -> anyhow::Result<()> {
//     let now = Instant::now();
//     // Run the model builder.
//     let addrs = parse_addresses_from_options(options.move_named_address_values.clone())?;

//     let specs = parse_specs_from_options(options.spec_sources.clone())?;

//     // This isn't the blackbox backprogagating tool.
//     // Just a bytecode only prover.
//     let env = run_bytecode_model_builder_with_specs()


//     // run_move_prover_with_model(&env, error_writer, options, Some(now))
// }


// pub fn parse_specs_from_options(
//     spec_sources: Vec<String>
// ) -> anyhow::Result<BTreeMap<String, Spec>> {
//     spec_sources
//         .iter()
//         .map(|spec_source| parse_spec_from_source(spec_source))
//         .collect()
// }

// pub fn parse_spec_from_source(
//     spec_source: String
// ) -> anyhow::Result<Spec> {
    
// }