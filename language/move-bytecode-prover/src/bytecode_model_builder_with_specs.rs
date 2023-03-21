use move_model::{
    code_writer::CodeWriter, model::{
        GlobalEnv,
        Spec,
    }, 
    parse_addresses_from_options,
    run_bytecode_model_builder,
};,

use move_binary_format::{
    file_format::{
        CompiledModule,
    },
};

// Build a `GlobalEnv' from a collection of `CompiledModules`'s and `Spec`'s
// This assumed that specs have been parsed from a specification module
// in a `.spec` file.
// This allows for some extensibility regarding `Spec`s. They do not have to
// originate from a file.

// ModuleData is taken care of by run_bytecode_model_builder_with_specs
pub fn run_bytecode_model_builder_with_specs(
    modules: impl IntoIterator<Item = &'a CompiledModule>,
    specs: impl IntoIterator<Item = &'a Spec>
) {


}

