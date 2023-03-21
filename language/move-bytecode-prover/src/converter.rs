use std::default;

use move_binary_format::{file_format as F, access::ModuleAccess};
// use move_model::ast as M;
// use move_compiler::shared::{
//     unique_map::UniqueMap,
//     unique_set::Uniqueset,
// };
use move_compiler::expansion::ast as E;
use move_compiler::parser::ast as P;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use move_command_line_common::{
    files::FileHash,
    address::NumericalAddress
};
use move_core_types::identifier::Identifier;
use move_compiler::shared::{Name};

// use std::format;

// // convert Vec<file_format::StructDefinition> into UniqueMap<StructName, StructDefinition>
// fn expansion_struct_defs_from_compiled_module(
//     compiled_module: &F::CompiledModule
// ) -> UniqueMap<E::StructName, E::StructDefinition> {

//     for (i, def) in m.struct_defs().iter().enumerate() {
//         let def_idx = StructDefinitionIndex(i as u16);
//         let name = m.identifier_at(m.struct_handle_at(def.struct_handle).name);
//         let symbol = env.symbol_pool().make(name.as_str());
//         let struct_id = StructId::new(symbol);
//         let data = env.create_move_struct_data(
//             m,
//             def_idx,
//             symbol,
//             Loc::default(),
//             Vec::default(),
//             Spec::default(),
//         );
//         module_data.struct_data.insert(struct_id, data);
//         module_data.struct_idx_to_id.insert(def_idx, struct_id);
//     }
    
//     let expansion_struct_defs = UniqueMap::new();

//     for (i, binary_struct_def) in compiled_module.struct_defs().iter().enumerate() {

//         let binary_struct_def_idx = F::StructDefinitionIndex(i as u16);
//         let name = m.identifier_at(m.struct_handle_at(binary_struct_def.struct_handle).name);

//         // Abilities
//         let abilities = expansion_ability_set_from(&binary_struct_def.abilities);

//         // Field Information
//         let fields = match binary_struct_def.field_information {
//             Native => E::StructFields::Native(Loc::default()),
//             Declared(field_defs) => {
//                 let struct_fields = UniqueMap::new();

//                 for (idx, field_def) in field_defs.iter().enumerate() {
//                     let name = compiled_module.identifier_at(field_def.name);
//                     let symbol = env.symbol_pool().make(name.as_str());
//                     struct_fields.add(
//                         P::Field(
//                             sp( , symbol);

//                             // location::Loc does not have a default while model::loc does.
//                             // the Loc used here is location::Loc
//                             Spanned::new(
//                                 Loc::default(),
//                                 symbol
//                             )
//                         ),
//                         (idx, ) // E::Type
//                     );
//                 }

//                 E::StructFields::Defined(struct_fields)
//             },
//         };

//         // Struct Type Parameters
//         let type_parameters = Vec::new();

//         for (i, binary_type_parameter) in binary_struct_def.type_parameters.iter().enumerate() {
//             type_parameters.push(
//                 E::StructTypeParameter{
//                     name: env.symbol_pool.make(
//                             format!(
//                                 "{}TypeParameter{}"
//                                 , name.as_str()
//                                 , i)
//                         ),
//                     constraints: expansion_ability_set_from(&binary_type_parameter.constraints),
//                     is_phantom: binary_type_parameter.is_phantom
//                 }
//             );
//         }

//         let model_struct_def = E::StructDefinition {
//             attributes: UniqueMap::new::<E::AttributeName, E::Attribute>(),
//             loc: Loc::default(),
//             abilities,
//             type_parameters,
//             fields,
//         };

//         expansion_struct_defs.add(model_struct_def);
//     }
// }

// fn expansion_ability_set_from(ability_set: &F::AbilitySet) -> E::AbilitySet {
//     match ability_set {
//         F::AbilitySet::EMPTY => E::AbilitySet::empty(),
//         F::AbilitySet::PRIMITIVES => E::AbilitySet::primitives(),
//         F::AbilitySet::REFERENCES => E::AbilitySet::references(),
//         F::AbilitySet::SIGNER => E::AbilitySet::signer(),
//         F::AbilitySet::COLLECTION => E::AbilitySet::collection(),
//     }
// }

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

// Decide between E::Type (Spanned<E::Type_>) and E::Type_
fn type_from(signature_token: &F::SignatureToken, compiled_module: &F::CompiledModule) -> E::Type {
    use E::Type_ as ET;
    use E::ModuleAccess_ as EMA;
    use F::SignatureToken as FST;

    match signature_token {
        FST::Address => sp(default_loc(), builtin_type(ADDRESS)),
        FST::Signer => sp(default_loc(), builtin_type(SIGNER)),
        FST::U8 => sp(default_loc(), builtin_type(U_8)),
        FST::U16 => sp(default_loc(), builtin_type(U_16)),
        FST::U32 => sp(default_loc(), builtin_type(U_32)),
        FST::U64 => sp(default_loc(), builtin_type(U_64)),
        FST::U128 => sp(default_loc(), builtin_type(U_128)),
        FST::U256 => sp(default_loc(), builtin_type(U_256)),
        FST::Bool => sp(default_loc(), builtin_type(BOOL)),
        FST::Vector(sig_tok) => 
            sp(
                default_loc(),
                ET::Apply(
                    sp(
                        default_loc(),
                        EMA::Name(
                            sp(
                                default_loc(), 
                                Symbol::from("vector")
                            ) 
                        )
                    ), 
                    vec![type_from(sig_tok, compiled_module)]
                )
            ),
        FST::Struct(idx) => {
            let struct_handle = compiled_module.struct_handle_at(*idx);

            // expansion::ast::ModuleIdentifier
            let module_handle = compiled_module.module_handle_at(struct_handle.module);
            let module_identifier = compiled_module.identifier_at(module_handle.name);

            let module_account_address = compiled_module.address_identifier_at(module_handle.address_identifier);

            let module_address = E::Address::Numerical(
                None,
                sp(
                    default_loc(),
                    NumericalAddress::new(module_account_address, P::NumberFormat::Hex)
                )
            );

            let module_ident = sp(
                default_loc(),
                E::ModuleIdent_{
                    address: module_address,
                    module: P::ModuleName(
                        sp(
                            default_loc(),
                            module_identifier.as_str()
                        )
                    )
                }
            );

            // Name (of the struct)
            let struct_identifier = compiled_module.identifier_at(struct_handle.name);
            let struct_name = sp(
                default_loc(),
                Symbol::from(struct_identifier.as_str())
            );

            let module_access = sp(
                default_loc(),
                EMA::ModuleAccess(module_ident, struct_name)
            );

            sp(
                default_loc(),
                ET::Apply(
                    module_access,
                    vec![]
                )
            )
        },
        FST::StructInstantiation(struct_handle_idx, sig_toks) => {
            
        },
        FST::Reference(sig_tok) => sp(default_loc(), type_from(sig_tok, compiled_module)),
        FST::MutableReference(sig_tok) => sp(default_loc(), type_from(sig_tok, compiled_module)),
        FST::TypeParameter(TypeParameterIndex) => ,
    }
}

// An intrinsic type holds 
fn builtin_type(type_name: &str) -> E::Type_ {
    use E::Type_ as ET;
    use E::ModuleAccess_ as EMA;

    let module_access = sp(
        default_loc(),
        EMA::Name(
            sp(
                default_loc(),
                Symbol::from(type_name)
            )
        )
    );

    ET::Apply(
        module_access,
        vec![]
    )
}

fn default_loc() -> Loc {
    Loc::new(FileHash::new("default"), 0, 0)
}

// #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_loc() {
        assert_eq!(
            Loc::new(FileHash::new("default"), 0, 0),
            default_loc()
        );
    }

    #[test]
    fn create_builtin_type() {
        use E::Type_ as ET;
        use E::ModuleAccess_ as EMA;
        
        let module_access = sp(
            default_loc(),
            EMA::Name(
                sp(
                    default_loc(),
                    Symbol::from("int")
                )
            )
        );
    
        let type_builtin_int = ET::Apply(
            sp(
                default_loc(),
                EMA::Name(
                    sp(
                        default_loc(),
                        Symbol::from("int")
                    )
                )
            ),
            vec![]
        );

        assert_eq!(type_builtin_int, builtin_type("int"));
    }

    #[test]
    fn symbol_from_identifier_reference() {
        let identifier = Identifier::new("SomeModule").unwrap();

        assert_eq!(Symbol::from("SomeModule"), Symbol::from((&identifier).as_ident_str().as_str()));
    }
}