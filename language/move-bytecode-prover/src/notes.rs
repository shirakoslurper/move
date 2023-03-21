//
// file_format::StructDefinition
//

pub struct StructDefinition {
    /// The `StructHandle` for this `StructDefinition`. This has the name and the abilities
    /// for the type.
    pub struct_handle: StructHandleIndex,
    /// Contains either
    /// - Information indicating the struct is native and has no accessible fields
    /// - Information indicating the number of fields and the start `FieldDefinition`s
    pub field_information: StructFieldInformation,
}

pub struct StructHandle {
    /// The module that defines the type.
    pub module: ModuleHandleIndex,
    /// The name of the type.
    pub name: IdentifierIndex,
    /// Contains the abilities for this struct
    /// For any instantiation of this type, the abilities of this type are predicated on
    /// that ability being satisfied for all type parameters.
    pub abilities: AbilitySet,
    /// The type formals (identified by their index into the vec)
    pub type_parameters: Vec<StructTypeParameter>,
}

pub enum StructFieldInformation {
    Native,
    Declared(Vec<FieldDefinition>),
}

pub struct FieldDefinition {
    /// The name of the field.
    pub name: IdentifierIndex,
    /// The type of the field.
    pub signature: TypeSignature,
}

pub struct TypeSignature(pub SignatureToken);

// 351  * `SIGNATURES`: The set of signatures in this binary. A signature is a
// 352: vector of [Signature Tokens](#SignatureTokens), so every signature will carry
// 353  the length (in ULEB128 form) followed by the Signature Tokens.

// 360  
// 361:     * `type`: the [Signature Token](#SignatureTokens) (type) of the value that follows
// 362      * `length`: the length of the serialized value in bytes

// 388              name of the field
// 389:             * `field type`: [SignatureToken](#SignatureTokens) - the type of
// 390              the field

// 462  
// 463: SignatureTokens
// 464  
// 465: A `SignatureToken` is 1 byte, and it is one of:
// 466  

// 471  * `0x5`: `ADDRESS` - an `AccountAddress` in the chain, may be a 16, 20, or 32 byte value
// 472: * `0x6`: `REFERENCE` - a reference; must be followed by another SignatureToken
// 473  representing the type referenced
// 474  * `0x7`: `MUTABLE_REFERENCE` - a mutable reference; must be followed by another
// 475: SignatureToken representing the type referenced
// 476  * `0x8`: `STRUCT` - a structure; must be followed by the index into the

// 480  The index is in ULEB128 form
// 481: * `0xA`: `VECTOR` - a vector - must be followed by another SignatureToken
// 482  representing the type of the vector

// 484  into the `STRUCT_HANDLES` table for the generic type of the instantiation, and a
// 485: vector describing the substitution types, that is, a vector of SignatureTokens
// 486  * `0xC`: `SIGNER` - a signer type, which is a special type for the VM

//
// E:: StructDefinition
//

pub struct StructDefinition {
    pub attributes: Attributes,
    pub loc: Loc,
    pub abilities: AbilitySet,
    pub type_parameters: Vec<StructTypeParameter>,
    pub fields: StructFields,
}

pub enum StructFields {
    Defined(Fields<Type>),
    Native(Loc),
}

// `Field` refers to a parser::ast::Field;
// 'usize' here refers to an index.
// We can just enumerate over our provided fields and use the index in this case.
// As for T, expansion::translate::StructFields uses the type_() function to 
// extract the type from the context and a Spanned<parser::ast::Type_>.

// We don't have a type_() function to convert between whatever
// file_format level type information there is in StructFieldInformation
// to E::Type_
pub type Fields<T> = UniqueMap<Field, (usize, T)>;


// Field wraps a Name like Field(Name)
// Name is from move_compiler::shared
pub type Name = Spanned<Symbol>;

// Symbol is from move_symbol_pool
// Spanned is from move_ir_types::location

pub struct Spanned<T> {
    pub loc: Loc,
    pub value: T,
}

impl<T> Spanned<T> {
    pub fn new(loc: Loc, value: T) -> Spanned<T> {
        Spanned { loc, value }
    }
}

/// The `Loc` struct is used to define a location in a file; where the file is considered to be a
/// vector of bytes, and the range for a given `Loc` is defined by start and end index into that
/// byte vector
pub struct Loc {
    /// The file the location points to
    file_hash: FileHash,
    /// The start byte index into file
    start: ByteIndex,
    /// The end byte index into file
    end: ByteIndex,
}


///
/// location::Loc
/// 
pub struct Loc {
    /// The file the location points to
    file_hash: FileHash,
    /// The start byte index into file
    start: ByteIndex,
    /// The end byte index into file
    end: ByteIndex,
}

// We should implement a default function of sorts using Loc::new().
// It would, however, become an issue if Loc is used as a key.
// In which case we would have to defer to an increasing count of sorts
// for the filehash and indices to guarantee uniqueness.

// move_command_line_common::files
pub struct FileHash(pub [u8; 32]);

impl FileHash {
    pub fn new(file_contents: &str) -> Self {
        Self(
            sha2::Sha256::digest(file_contents.as_bytes())
                .try_into()
                .expect("Length of sha256 digest must always be 32 bytes"),
        )
    }

    pub const fn empty() -> Self {
        Self([0; 32])
    }
}

pub type ByteIndex = u32;


// HLIR?
pub enum TypeName_ {
    Builtin(BuiltinTypeName),
    ModuleType(expansion::ast::ModuleIdent, parser::ast::StructName),
}

// expansion::ast
pub struct ModuleIdent_ {
    pub address: Address,
    pub module: parser::ast::ModuleName,
}

// We can instantiate ModuleName with ModuleName(Identifier). NOPE. This is move_ir_types::ModuleName


// file_format
pub struct ModuleHandle {
    /// Index into the `AddressIdentifierIndex`. Identifies module-holding account's address.
    pub address: AddressIdentifierIndex,
    /// The name of the module published in the code section for the account in `address`.
    pub name: IdentifierIndex,
}

pub struct StructHandle {
    /// The module that defines the type.
    pub module: ModuleHandleIndex,
    /// The name of the type.
    pub name: IdentifierIndex,
    /// Contains the abilities for this struct
    /// For any instantiation of this type, the abilities of this type are predicated on
    /// that ability being satisfied for all type parameters.
    pub abilities: AbilitySet,
    /// The type formals (identified by their index into the vec)
    pub type_parameters: Vec<StructTypeParameter>,
}