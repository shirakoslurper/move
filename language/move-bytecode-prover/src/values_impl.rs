
use move_core_types::{
    u256::U256,
    account_address::AccountAddress,
    value::MoveTypeLayout
};
use move_command_line_common::{
    address::NumericalAddress,
    parser::NumberFormat
};
use move_ir_types::location::*;
use serde::{
    Deserialize,
};
use move_binary_format::file_format as F;
use move_compiler::expansion::ast as E;

use anyhow::{Result, Context};

// // Using newtype pattern to add simple deserialization to expansion::ast::Value
// pub struct ExpansionValueWrapper(pub E::Value);


pub fn simple_deserialize(blob: &[u8], layout: &MoveTypeLayout) -> Option<E::Value> {
    bcs::from_bytes_seed(SeedWrapper { layout }, blob).ok()
}

// Bit hacky and not very idomatic
// Best would be to reimplement MoveTypeLayout with the base
// numerical types, bool, and Address + a Bytearray Variant but 
// we're simple gonna have constant_sig_token_to_layout return Signer
// on the Bytearray corresponding variants. 

// Our deserialize_constant() and DeserializeSeed impls
// are going to have to be aware of this, though. Signer will
// be thrown into the default case.

// TODO: Refactor. This is a terrible fix.

fn constant_sig_token_to_layout(constant_signature: &F::SignatureToken) -> Option<MoveTypeLayout> {
    use MoveTypeLayout as L;
    use F::SignatureToken as FST;

    Some(match constant_signature {
        FST::Bool => L::Bool,
        FST::U8 => L::U8,
        FST::U16 => L::U16,
        FST::U32 => L::U32,
        FST::U64 => L::U64,
        FST::U128 => L::U128,
        FST::U256 => L::U256,
        FST::Address => L::Address,
        _ => L::Signer,
    })
}

pub fn deserialize_constant(constant: &F::Constant) -> Option<E::Value> {
    let layout = constant_sig_token_to_layout(&constant.type_)?;
    let deserialized = simple_deserialize(&constant.data, &layout);

    // println!("Deserialized: {:#?}", deserialized);
    deserialized
}


// Using newtype pattern to add trait for MoveTypeLayout
pub struct SeedWrapper<L> {
    layout: L
}

impl<'d> serde::de::DeserializeSeed<'d> for SeedWrapper<&MoveTypeLayout> {
    type Value = E::Value;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D
    ) -> Result<Self::Value, D::Error> {
        use MoveTypeLayout as L;
        use E::Value_ as EV;
        Ok(
            Spanned::unsafe_no_loc(
                match self.layout {
                    L::Bool => EV::Bool(bool::deserialize(deserializer)?),
                    L::U8 => EV::U8(u8::deserialize(deserializer)?),
                    L::U16 => EV::U16(u16::deserialize(deserializer)?),
                    L::U32 => EV::U32(u32::deserialize(deserializer)?),
                    L::U64 => EV::U64(u64::deserialize(deserializer)?),
                    L::U128 => EV::U128(u128::deserialize(deserializer)?),
                    L::U256 => EV::U256(U256::deserialize(deserializer)?),
                    L::Address => {
                        // We can't instantiate a NumericalAddress with an AccountAddress
                        let account_address = AccountAddress::deserialize(deserializer)?;

                        EV::Address(
                            E::Address::Numerical(
                                None,
                                Spanned::unsafe_no_loc(
                                    NumericalAddress::new(
                                        account_address.into_bytes(),
                                        NumberFormat::Hex
                                    )
                                )
                            )
                        )
                    },
                    _ => {
                        EV::Bytearray(<Vec<u8>>::deserialize(deserializer)?)
                    },
                }
            )
        )
    }
}



// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn deserialize() -> Result<()> {

//         let value = 

//         Ok(())
//     }
// }
