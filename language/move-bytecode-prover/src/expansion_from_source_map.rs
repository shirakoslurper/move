use move_bytecode_source_map::{
    mapping::SourceMapping,
    source_map::{FunctionSourceMap, SourceName},
};
use move_binary_format::{file_format as F, access::ModuleAccess, constant};
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
    parser::NumberFormat, values
};
use move_core_types::identifier::{Identifier, IdentStr};
use move_compiler::shared::{Name};
use anyhow::{bail, Error, Result, Context};
use std::{format, collections::{BTreeMap, VecDeque}};

use crate::values_impl;

pub struct Deriver<'a> {
    pub source_mapper: SourceMapping<'a>, 
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

    // pub struct Constant {
    //     pub attributes: Attributes,
    //     pub loc: Loc,
    //     pub signature: Type,
    //     pub value: Exp,
    // }

    pub fn derive_constants(&self) -> Result<Vec<(P::ConstantName, E::Constant)>> {
        self
            .source_mapper
            .source_map
            .constant_map
            .iter()
            .map(|(name, table_idx)| {
                let name = P::ConstantName(
                    Spanned::unsafe_no_loc(name.0)
                );

                let constant =  self
                    .source_mapper
                    .bytecode
                    .constant_at(F::ConstantPoolIndex(table_idx.clone()));

                Ok((name, self.derive_constant(constant)?))
            })
            .collect::<Result<Vec<(P::ConstantName, E::Constant)>>>()
    }

    // Instead of deriving the values separately (constant.data), we derive the 
    // values from the whole constant as TypeSignature information
    // provides extra robustness when it comes to serialization/deserialization.

    pub fn derive_constant(&self, constant: &F::Constant) -> Result<E::Constant> {
        Ok(
            E::Constant {
                attributes: UniqueMap::new(),
                loc: no_loc(),
                signature: self.derive_type_from_sig_tok(
                    constant.type_.clone(),
                    &[]),
                value: Spanned::unsafe_no_loc(
                    E::Exp_::Value(
                        values_impl::deserialize_constant(constant).context("Failed")?
                    )
                )
            }
        )
    }

    // pub struct Function {
    //     pub attributes: Attributes,
    //     pub loc: Loc,
    //     pub visibility: Visibility,
    //     pub entry: Option<Loc>,
    //     pub signature: FunctionSignature,
    //     pub acquires: Vec<ModuleAccess>,
    //     pub body: FunctionBody,
    //     pub specs: BTreeMap<SpecId, SpecBlock>,
    // }

    pub fn derive_function_def(
        &self,
        function_source_map: &FunctionSourceMap,
        function: Option<(&F::FunctionDefinition, &F::FunctionHandle)>,
        name: &IdentStr,
        type_parameters: &[F::AbilitySet],
        parameters: F::SignatureIndex,
        code: Option<&F::CodeUnit>, // Not using atm
    ) -> Result<(P::FunctionName, E::Function)>{
        use E::ModuleAccess_ as EMA;

        debug_assert_eq!(
            function_source_map.parameters.len(),
            self.source_mapper.bytecode.signature_at(parameters).len(),
            "Arity mismatch between function source map and bytecode for function {}",
            name
        );

        let loc = function_source_map.definition_location;

        let entry = if function.map(|(f, _)| f.is_entry).unwrap_or(false) {
            Some(no_loc())
        } else {
            None
        };

        let visibility = match function {
            Some(function) => match function.0.visibility {
                F::Visibility::Private => E::Visibility::Internal,
                F::Visibility::Friend => E::Visibility::Friend(no_loc()),
                F::Visibility::Public => E::Visibility::Public(no_loc()),
            },
            None => E::Visibility::Internal,
        };

        // Vec<ModuleAccess>
        let acquires = match function {
            Some(function) => {
                function.0.acquires_global_resources
                    .iter()
                    .map(|struct_def_idx| {
                        let struct_def = self.source_mapper.bytecode.struct_def_at(*struct_def_idx)?;
                        Ok(self.module_access_from_struct_handle_index(&struct_def.struct_handle))
                    })
                    .collect::<Result<Vec<Spanned<EMA>>>>()
            },
            None => Ok(vec![])
        }?;

        let signature = self.derive_function_sig(
            function_source_map,
            function.expect("Not provided a function."),
            type_parameters,
            parameters
        );

        // TODO: Use CodeUnit?

        Ok(
            (
                P::FunctionName(
                    sp(loc, Symbol::from(name.as_str()))
                ),
                E::Function {
                    attributes: UniqueMap::new(),
                    loc,
                    visibility,
                    entry,
                    signature,
                    acquires,
                    body: Spanned::unsafe_no_loc(E::FunctionBody_::Defined(VecDeque::new())),
                    specs: BTreeMap::new(),
                }
            )
        )

    }

    fn derive_function_sig(
        &self,
        function_source_map: &FunctionSourceMap,
        function: (&F::FunctionDefinition, &F::FunctionHandle),
        type_parameters: &[F::AbilitySet],
        parameters: F::SignatureIndex
    ) -> E::FunctionSignature {
        
        let type_parameters = Self::derive_fun_type_formals(
            &function_source_map.type_parameters, 
            type_parameters
        );

        let parameters = self
            .source_mapper
            .bytecode
            .signature_at(parameters)
            .0
            .iter()
            .zip(&function_source_map.parameters)
            .map(|(sig_tok, (name, loc))| {
                (
                    P::Var(sp(loc.clone(), Symbol::from(name.clone()))),
                    self.derive_type_from_sig_tok(
                        sig_tok.clone(), 
                        &function_source_map.type_parameters
                    )
                )
            })
            .collect::<Vec<(P::Var, E::Type)>>();

        let return_sig = self
            .source_mapper
            .bytecode
            .signature_at(function.1.return_);

        let return_sig_toks = &return_sig
            .0;

        let return_loc = no_loc();

        let return_type = match return_sig_toks.len() {
            0 => sp(return_loc, E::Type_::Unit),
            _ => sp(
                return_loc,
                E::Type_::Multiple(
                    return_sig_toks
                        .iter()
                        .map(|sig_tok| {
                            self.derive_type_from_sig_tok(
                                sig_tok.clone(), 
                                &function_source_map.type_parameters
                            )
                        })
                        .collect::<Vec<E::Type>>()
                )
            ),
        };

        E::FunctionSignature {
            type_parameters,
            parameters,
            return_type,
        }
    }

    fn derive_fun_type_formals(
        source_map_ty_params: &[SourceName],
        abilities: &[F::AbilitySet],
    ) -> Vec<(Name, E::AbilitySet)> {

        source_map_ty_params
            .iter()
            .zip(abilities)
            .map(|((name, loc), ability_set)| {
                (
                    sp(*loc, Symbol::from(name.clone())),
                    if *ability_set == F::AbilitySet::EMPTY {
                        E::AbilitySet::empty()
                    } else {
                        Self::ability_set_from(*ability_set)
                    }
                )
            })
            .collect::<Vec<(Name, E::AbilitySet)>>()
    }

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
                no_loc()
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
            .map(|((name, loc), ty_param)| {
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
                    name: sp(*loc, Symbol::from(name.clone())),
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
                let source_name = type_param_context
                .get(ty_param_idx as usize)
                .expect("Failed to get type parameter from context with index.");

                let name = source_name
                    .0
                    .clone();
                
                let loc = source_name
                    .1
                    .clone();

                let module_access = Spanned::unsafe_no_loc(
                    EMA::Name(
                        Spanned::unsafe_no_loc(Symbol::from(name))
                    )
                );

                sp(
                    loc,
                    ET::Apply(
                        module_access,
                        vec![]
                    )
                )
            },
        }
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

    fn ability_set_from(ability_set: F::AbilitySet) -> E::AbilitySet {
        match ability_set {
            F::AbilitySet::EMPTY => E::AbilitySet::empty(),
            F::AbilitySet::PRIMITIVES => E::AbilitySet::primitives(no_loc()),
            F::AbilitySet::REFERENCES => E::AbilitySet::references(no_loc()),
            F::AbilitySet::SIGNER => E::AbilitySet::signer(no_loc()),
            _ => panic!("No such ability set")
            // F::AbilitySet::VECTOR => E::AbilitySet::collection(no_loc()),
        }
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

fn no_loc() -> Loc {
    Loc::new(FileHash::empty(), 0, 0)
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
    fn derive_struct_definition_test() -> Result<()> {
        let no_loc = no_loc();
        let bytecode_bytes = fs::read("/Users/asaphbay/research/move/language/move-bytecode-prover/sample-bytecode/amm.mv").expect("Unable to read bytecode file");
        let module = F::CompiledModule::deserialize(&bytecode_bytes)?;
        let bytecode = BinaryIndexedView::Module(&module);
        let source_mapping = SourceMapping::new_from_view(bytecode, no_loc)?;

        let sample_file_deriver = Deriver::new(source_mapping);
        let (name, struct_definition) = sample_file_deriver.derive_struct_def(F::StructDefinitionIndex(3_u16))?;
        
        ast_debug::print(&(name, &struct_definition));
        
        Ok(())
    }

    #[test]
    fn derive_function_definition_test() -> Result<()> {
        let no_loc = no_loc();
        let bytecode_bytes = fs::read("/Users/asaphbay/research/move/language/move-bytecode-prover/sample-bytecode/amm.mv").expect("Unable to read bytecode file");
        let module = F::CompiledModule::deserialize(&bytecode_bytes)?;
        let bytecode = BinaryIndexedView::Module(&module);
        let source_mapping = SourceMapping::new_from_view(bytecode, no_loc)?;

        let deriver = Deriver::new(source_mapping);

        let function_defs = match deriver.source_mapper.bytecode {
            BinaryIndexedView::Script(script) => {
                panic!("AGHHH");
            }
            BinaryIndexedView::Module(module) => (0..module.function_defs.len())
                .map(|i| {
                    let function_definition_index = F::FunctionDefinitionIndex(i as F::TableIndex);
                    let function_definition = deriver.get_function_def(function_definition_index)?;
                    let function_handle = deriver
                        .source_mapper
                        .bytecode
                        .function_handle_at(function_definition.function);
                    deriver.derive_function_def(
                        deriver.source_mapper
                            .source_map
                            .get_function_source_map(function_definition_index)?,
                        Some((function_definition, function_handle)),
                        deriver.source_mapper
                            .bytecode
                            .identifier_at(function_handle.name),
                        &function_handle.type_parameters,
                        function_handle.parameters,
                        function_definition.code.as_ref(),
                    )
                })
                .collect::<Result<Vec<(P::FunctionName, E::Function)>>>()?,
        };
        
        ast_debug::print(&(function_defs.get(9).context("No name.")?.0, &function_defs.get(9).context("No definition.")?.1));
        
        Ok(())
    }

    #[test]
    fn derive_constants_test() -> Result<()> {
        let no_loc = no_loc();
        let bytecode_bytes = fs::read("/Users/asaphbay/research/move/language/move-bytecode-prover/sample-bytecode/amm.mv").expect("Unable to read bytecode file");
        let module = F::CompiledModule::deserialize(&bytecode_bytes)?;
        let bytecode = BinaryIndexedView::Module(&module);
        let source_mapping = SourceMapping::new_from_view(bytecode, no_loc)?;

        let deriver = Deriver::new(source_mapping);

        println!("Number of constants: {}", deriver.source_mapper.source_map.constant_map.keys().len());

        // for (name, table_idx) in deriver.source_mapper.source_map.constant_map.iter() {
        //     println!("{:#?}: {:#?}", 
        //     name,
        //     deriver
        //         .source_mapper
        //         .bytecode
        //         .constant_at(F::ConstantPoolIndex(table_idx.clone())));
        // }

        let constants = deriver.derive_constants()?;

        for (name, constant) in constants {
            ast_debug::print(&(name, &constant));
        }

        Ok(())
    }

}