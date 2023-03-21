use move_bytecode_source_map::{
    mapping::SourceMapping,
    source_map::{FunctionSourceMap, SourceName},
};
use move_binary_format::{file_format as F, access::ModuleAccess};
// use move_model::ast as M;
use move_compiler::shared::{
    unique_map::UniqueMap,
    // unique_set::Uniqueset,
};
use move_compiler::expansion::ast as E;
use move_compiler::parser::ast as P;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use move_command_line_common::{
    files::FileHash,
    address::NumericalAddress,
    parser::NumberFormat
};
use move_core_types::identifier::{Identifier, IdentStr};
use move_compiler::shared::{Name};
use anyhow::{bail, Error, Result};
use std::format;

pub struct Deriver<'a> {
    source_mapper: SourceMapping<'a>, 
}

impl<'a> Deriver<'a> {
    pub fn new(source_mapper: SourceMapping<'a>) -> Self {
        Self {
            source_mapper
        }
    }


    //***************************************************************************
    // Helpers
    //***************************************************************************

    fn get_function_def(
        &self,
        function_definition_index: F::FunctionDefinitionIndex,
    ) -> Result<&F::FunctionDefinition> {
        if function_definition_index.0 as usize
            >= self
                .source_mapper
                .bytecode
                .function_defs()
                .map_or(0, |f| f.len())
        {
            bail!("Invalid function definition index supplied when marking function")
        }
        match self
            .source_mapper
            .bytecode
            .function_def_at(function_definition_index)
        {
            Ok(definition) => Ok(definition),
            Err(err) => Err(Error::new(err)),
        }
    }

    fn get_struct_def(
        &self,
        struct_definition_index: F::StructDefinitionIndex,
    ) -> Result<&F::StructDefinition> {
        if struct_definition_index.0 as usize
            >= self
                .source_mapper
                .bytecode
                .struct_defs()
                .map_or(0, |d| d.len())
        {
            bail!("Invalid struct definition index supplied when marking struct")
        }
        match self
            .source_mapper
            .bytecode
            .struct_def_at(struct_definition_index)
        {
            Ok(definition) => Ok(definition),
            Err(err) => Err(Error::new(err)),
        }
    }

    //***************************************************************************
    // Derivers
    //***************************************************************************

    // pub fn derive_function_def(
    //     &self,
    //     function_source_map: &FunctionSourceMap,
    //     function: Option<(&FunctionDefinition, &FunctionHandle)>,
    //     name: &IdentStr,
    //     type_parameters: &[AbilitySet],
    //     parameters: SignatureIndex,
    //     code: Option<&CodeUnit>,
    // ) -> Result<>{
    //     debug_assert_eq!(
    //         function_source_map.parameters.len(),
    //         self.source_mapper.bytecode.signature_at(parameters).len(),
    //         "Arity mismatch between function source map and bytecode for function {}",
    //         name
    //     );

    // }

    fn derive_struct_def(
        &self,
        struct_def_idx: F::StructDefinitionIndex
    ) -> Result<(P::StructName, E::StructDefinition)>{
        let struct_definition = self.get_struct_def(struct_def_idx)?;
        let struct_handle = self
            .source_mapper
            .bytecode
            .struct_handle_at(struct_definition.struct_handle);
        let struct_source_map = self
            .source_mapper
            .source_map
            .get_struct_source_map(struct_def_idx)?;

        let loc = struct_source_map.definition_location;

        let field_info: Option<Vec<(&IdentStr, &F::TypeSignature)>> =
            match &struct_definition.field_information {
                F::StructFieldInformation::Native => None,
                F::StructFieldInformation::Declared(fields) => Some(
                    fields
                        .iter()
                        .map(|field_definition| {
                            let type_sig = &field_definition.signature;
                            let field_name = self
                                .source_mapper
                                .bytecode
                                .identifier_at(field_definition.name);
                            (field_name, type_sig)
                        })
                        .collect(),
                ),
            };

        let abilities = if struct_handle.abilities == F::AbilitySet::EMPTY {
            E::AbilitySet::empty()
        } else {
            let abilities_vec: Vec<_> = struct_handle
                .abilities
                .into_iter()
                .map(Self::ability_from)
                .collect();
            E::AbilitySet::from_abilities(abilities_vec).expect("Failed to make struct ability set")
        };

        let name = self
            .source_mapper
            .bytecode
            .identifier_at(struct_handle.name)
            .to_string();

        let type_parameters = Self::derive_struct_type_formals(
            &struct_source_map.type_parameters,
            &struct_handle.type_parameters,
        );

        let fields = match field_info {
            None => E::StructFields::Native(
                Loc::new(FileHash::empty(), 0, 0)
            ),
            Some(field_info) => {
                let mut field_map: E::Fields<E::Type> = UniqueMap::new();
                for (idx, (name, ty_sig)) in field_info.iter().enumerate() {
                    if let Err(_) = field_map.add(
                        P::Field(
                            Spanned::unsafe_no_loc(Symbol::from(name.as_str()))
                        ),
                        (idx, self.derive_type_from_sig_tok(ty_sig.0.clone(), &struct_source_map.type_parameters))
                    ) {
                        panic!("Failed field inserttion into field map.");
                    };
                };
                E::StructFields::Defined(field_map)
            }
        };

        Ok(
            (
                P::StructName(
                    Spanned::unsafe_no_loc(Symbol::from(name.as_str()))
                ), 
                E::StructDefinition {
                    attributes: UniqueMap::new(),
                    loc,
                    abilities,
                    type_parameters,
                    fields,
                }
            )
        )
    }

    fn derive_struct_type_formals(
        source_map_ty_params: &[SourceName],
        type_parameters: &[F::StructTypeParameter],
    ) -> Vec<E::StructTypeParameter> {
        let ty_params = source_map_ty_params
            .iter()
            .zip(type_parameters)
            .map(|((name, _), ty_param)| {

                let ability_set = if ty_param.constraints == F::AbilitySet::EMPTY {
                    E::AbilitySet::empty()
                } else {
                    let abilities = ty_param
                        .constraints
                        .into_iter()
                        .map(Self::ability_from)
                        .collect::<Vec<P::Ability>>();

                    E::AbilitySet::from_abilities(abilities).expect("Failed to make ability set.")
                };

                E::StructTypeParameter{
                    is_phantom: ty_param.is_phantom,
                    name: Spanned::unsafe_no_loc(Symbol::from(name.clone())),
                    constraints: ability_set
                }

            })
            .collect();

        ty_params
    }

    pub const ADDRESS: &'static str = "address";
    pub const SIGNER: &'static str = "signer";
    pub const U_8: &'static str = "u8";
    pub const U_16: &'static str = "u16";
    pub const U_32: &'static str = "u32";
    pub const U_64: &'static str = "u64";
    pub const U_128: &'static str = "u128";
    pub const U_256: &'static str = "u256";
    pub const BOOL: &'static str = "bool";
    pub const VECTOR: &'static str = "vector";

    fn derive_type_from_sig_tok(
        &self,
        sig_tok: F::SignatureToken,
        type_param_context: &[SourceName]
    ) -> E::Type {
        use E::Type_ as ET;
        use E::ModuleAccess_ as EMA;
        use F::SignatureToken as FST;

        match sig_tok {
            FST::Address => Spanned::unsafe_no_loc(Self::builtin_type( Self::ADDRESS)),
            FST::Signer => Spanned::unsafe_no_loc(Self::builtin_type(Self::SIGNER)),
            FST::U8 => Spanned::unsafe_no_loc(Self::builtin_type(Self::U_8)),
            FST::U16 => Spanned::unsafe_no_loc(Self::builtin_type(Self::U_16)),
            FST::U32 => Spanned::unsafe_no_loc(Self::builtin_type(Self::U_32)),
            FST::U64 => Spanned::unsafe_no_loc(Self::builtin_type(Self::U_64)),
            FST::U128 => Spanned::unsafe_no_loc(Self::builtin_type(Self::U_128)),
            FST::U256 => Spanned::unsafe_no_loc(Self::builtin_type(Self::U_256)),
            FST::Bool => Spanned::unsafe_no_loc(Self::builtin_type(Self::BOOL)),
            FST::Vector(sig_tok) => 
                Spanned::unsafe_no_loc(
                    ET::Apply(
                        Spanned::unsafe_no_loc(
                            EMA::Name(
                                Spanned::unsafe_no_loc(Symbol::from("vector"))
                            )
                        ), 
                        vec![self.derive_type_from_sig_tok((*sig_tok).clone(), type_param_context)]
                    )
                ),
            FST::Struct(struct_handle_idx) => {
                let module_access = self.module_access_from_struct_handle_index(&struct_handle_idx);
                Spanned::unsafe_no_loc(
                    ET::Apply(
                        module_access,
                        vec![]
                    )
                )
            },
            FST::StructInstantiation(struct_handle_idx, sig_toks) => {
                let types = sig_toks
                    .iter()
                    .map(|sig_tok| self.derive_type_from_sig_tok((*sig_tok).clone(), type_param_context))
                    .collect();
                
                let module_access = self.module_access_from_struct_handle_index(&struct_handle_idx);
                Spanned::unsafe_no_loc(
                    ET::Apply(
                        module_access,
                        types
                    )
                )
            },
            FST::Reference(sig_tok) => self.derive_type_from_sig_tok(*sig_tok, type_param_context),
            FST::MutableReference(sig_tok) => self.derive_type_from_sig_tok(*sig_tok, type_param_context),
            FST::TypeParameter(ty_param_idx) => {
                let name = type_param_context
                    .get(ty_param_idx as usize)
                    .expect("Failed to get type parameter from context with index.")
                    .0
                    .clone()
                    ;

                let module_access = Spanned::unsafe_no_loc(
                    EMA::Name(
                        Spanned::unsafe_no_loc(Symbol::from(name))
                    )
                );

                Spanned::unsafe_no_loc(
                    ET::Apply(
                        module_access,
                        vec![]
                    )
                )
            },
        }
    }

    fn struct_type_formals_from(names: Vec<&str>, compiled_ty_params: Vec<F::StructTypeParameter>) -> Vec<E::StructTypeParameter>{
        assert_eq!(names.len(), compiled_ty_params.len(), "Lengths of names and compiled type parameters do not match");
        names
            .iter()
            .zip(compiled_ty_params)
            .map(|(name, ty_param)| {

                let ability_set = if ty_param.constraints == F::AbilitySet::EMPTY {
                    E::AbilitySet::empty()
                } else {
                    let abilities = ty_param
                        .constraints
                        .into_iter()
                        .map(Self::ability_from)
                        .collect::<Vec<P::Ability>>();

                    E::AbilitySet::from_abilities(abilities).expect("Failed to make ability set.")
                };

                E::StructTypeParameter{
                    is_phantom: ty_param.is_phantom,
                    name: Spanned::unsafe_no_loc(Symbol::from(name.clone())),
                    constraints: ability_set
                }

            })
            .collect()
    }

    // Helpers

    fn module_access_from_struct_handle_index(
        &self,
        struct_handle_idx: &F::StructHandleIndex
    ) -> E::ModuleAccess {
        use E::ModuleAccess_ as EMA;

        let struct_handle = self.source_mapper.bytecode.struct_handle_at(*struct_handle_idx);

        // expansion::ast::ModuleIdentifier
        let module_handle = self.source_mapper.bytecode.module_handle_at(struct_handle.module);
        let module_identifier = self.source_mapper.bytecode.identifier_at(module_handle.name);

        let module_account_address = self.source_mapper.bytecode.address_identifier_at(module_handle.address);

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
        
        // Name (of the struct)
        let struct_identifier = self.source_mapper.bytecode.identifier_at(struct_handle.name);
        let struct_name = Spanned::unsafe_no_loc(
            Symbol::from(struct_identifier.as_str())
        );

        Spanned::unsafe_no_loc(
            EMA::ModuleAccess(module_ident, struct_name)
        )
    }

    fn ability_from(compiled_ability: F::Ability) -> P::Ability {
        Spanned::unsafe_no_loc(
            match compiled_ability {
                F::Ability::Copy => P::Ability_::Copy,
                F::Ability::Drop => P::Ability_::Drop,
                F::Ability::Store => P::Ability_::Store,
                F::Ability::Key => P::Ability_::Key,
            }
        )
    }

    fn builtin_type(type_name: &str) -> E::Type_ {
        use E::Type_ as ET;
        use E::ModuleAccess_ as EMA;

        let module_access = 
            Spanned::unsafe_no_loc(
                EMA::Name(
                    Spanned::unsafe_no_loc(Symbol::from(type_name))
                )
            );

        ET::Apply(
            module_access,
            vec![]
        )
    }

}



#[cfg(test)]
mod tests {
    use move_bytecode_source_map::utils::source_map_from_file;
    use move_binary_format::binary_views::BinaryIndexedView;
    use super::*;
    use std::{fs, path::Path};
    use move_compiler::shared::ast_debug;

    #[test]
    fn ability() {
        assert_eq!(
            Spanned::unsafe_no_loc(P::Ability_::Copy),
            Deriver::ability_from(F::Ability::Copy)
        );
    }

    #[test]
    fn struct_type_formals() {
        assert_eq!(
            vec![
                E::StructTypeParameter{
                    is_phantom: true,
                    name: Spanned::unsafe_no_loc(Symbol::from("test")),
                    constraints: E::AbilitySet::all(Loc::new(FileHash::empty(), 0, 0)),
                }
            ],
            Deriver::struct_type_formals_from(
                vec![
                    "test"
                ],
                vec![
                    F::StructTypeParameter{
                        constraints: F::AbilitySet::ALL,
                        is_phantom: true
                    }
                ]
            )
        );
    }

    #[test]
    fn derive_struct_definition() -> Result<()> {
        let no_loc = Spanned::unsafe_no_loc(()).loc;
        let bytecode_bytes = fs::read("/Users/asaphbay/research/move/language/move-bytecode-prover/sample-bytecode/amm.mv").expect("Unable to read bytecode file");
        let module = F::CompiledModule::deserialize(&bytecode_bytes)?;
        let bytecode = BinaryIndexedView::Module(&module);
        let source_mapping = SourceMapping::new_from_view(bytecode, no_loc)?;

        let sample_file_deriver = Deriver::new(source_mapping);
        let (name, struct_definition) = sample_file_deriver.derive_struct_def(F::StructDefinitionIndex(3_u16))?;
        
        ast_debug::print(&(name, &struct_definition));
        
        Ok(())
    }

}