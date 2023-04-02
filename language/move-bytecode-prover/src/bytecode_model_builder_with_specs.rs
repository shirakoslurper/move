use std::collections::BTreeMap;

use move_model::{
    builder::{
        module_builder::ModuleBuilder,
        model_builder::ModelBuilder,
    },
    model::{ModuleId, GlobalEnv},
    ast::ModuleName,
    run_spec_simplifier
};

use move_bytecode_source_map::{
    mapping::SourceMapping,
};

use move_binary_format::{
    binary_views::BinaryIndexedView,
    file_format as F
};

use move_compiler::shared::unique_map::UniqueMap;

use move_compiler::{
    expansion::ast as E,
    parser::ast as P,
    compiled_unit::FunctionInfo
};

use move_ir_types::location::*;
use move_symbol_pool::*;

use anyhow::{anyhow, bail, Error, Result, Context};

use move_command_line_common::{
    address::NumericalAddress,
    parser::NumberFormat,
};

use crate::expansion_from_source_map::*;


// Build a `GlobalEnv' from a collection of `CompiledModules`'s and `Spec`'s
// This assumed that specs have been parsed from a specification module
// in a `.spec` file.
// This allows for some extensibility regarding `Spec`s. They do not have to
// originate from a file.

// // ModuleData is taken care of by run_bytecode_model_builder_with_specs
// pub fn run_bytecode_model_builder_with_specs(
//     modules: impl IntoIterator<Item = &'a CompiledModule>,
//     specs: impl IntoIterator<Item = &'a Spec>
// ) {


// }

pub fn run_spec_checker(env: &mut GlobalEnv, compiled_module: &F::CompiledModule) -> Result<()> {

    let bytecode = BinaryIndexedView::Module(compiled_module);

    let source_mapping = SourceMapping::new_from_view(bytecode, no_loc())?;

    let expansion_deriver = Deriver::new(source_mapping);

    let module = match expansion_deriver.derive()? {
        Derived::Module(expanded_module) => {
            let module_handle = expansion_deriver
                .source_mapper
                .bytecode
                .module_handle_at(
                    expansion_deriver
                        .source_mapper
                        .bytecode
                        .self_handle_idx()
                        .context("Failed to find module's own ModuleHandleIndex")?
                );

            let module_identifier = expansion_deriver.source_mapper.bytecode.identifier_at(module_handle.name);

            let module_account_address = expansion_deriver.source_mapper.bytecode.address_identifier_at(
                module_handle.address
            );

            let module_address = E::Address::Numerical(
                None,
                Spanned::unsafe_no_loc(
                    NumericalAddress::new((*module_account_address).into_bytes(), NumberFormat::Hex)
                )
            );

            let module_ident = Spanned::unsafe_no_loc(
                E::ModuleIdent_{
                    address: module_address,
                    module: P::ModuleName(
                        Spanned::unsafe_no_loc(
                            Symbol::from(module_identifier.as_str())
                        )
                    )
                }
            );

            let source_map = expansion_deriver
                .source_mapper
                .source_map
                .clone();

            // Function Infos
            let mut function_infos: UniqueMap<P::FunctionName, FunctionInfo> = UniqueMap::new();

            for (loc, symbol, _) in expanded_module.functions.iter() {
                if let Err(_) = function_infos.add(
                    P::FunctionName(
                        sp(loc, symbol.clone())
                    ),
                    FunctionInfo {
                        spec_info: BTreeMap::new(),
                        parameters: vec![],
                        attributes: UniqueMap::new(),
                    }
                ) {
                    return Err(anyhow!("Failed field insertion into map."));
                };
            }

            (
                module_ident,
                expanded_module,
                compiled_module,
                source_map,
                function_infos
            )
        },
        Derived::Script(_) => return Err(anyhow!("Derived a script. TODO: Handle scripts.")),
    };

    let (
        module_ident,
        expanded_module,
        compiled_module,
        source_map,
        function_infos
    ) = module;

    let mut builder = ModelBuilder::new(env);

    let loc = builder.to_loc(&expansion_deriver.source_mapper.source_map.definition_location);
    let addr_bytes = builder.resolve_address(&loc, &module_ident.value.address);
    let module_name = ModuleName::from_address_bytes_and_name(
        addr_bytes,
        builder
            .env
            .symbol_pool()
            .make(&module_ident.value.module.0.value),
    );

    let module_id = ModuleId::new(0);

    let mut module_translator = ModuleBuilder::new(&mut builder, module_id, module_name);
    module_translator.translate(
        loc,
        expanded_module,
        compiled_module.clone(),
        source_map,
        function_infos,
    );

    builder.populate_env();

    builder.warn_unused_schemas();

    run_spec_simplifier(env);
    
    Ok(())
}

