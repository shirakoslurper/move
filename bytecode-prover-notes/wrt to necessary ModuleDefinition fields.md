It seems beside `specs` and `attributes` (because they are for non_published code), we will have to build out these fields from elements of `CompiledModule`:
- `structs`
- `functions`
- `constants`

There are 3 relevant fields in `binary_view::file_format::CompiledModule`:
```
pub struct CompiledModule {
...
pub constant_pool: ConstantPool,
...
pub struct_defs: Vec<StructDefinition>,
pub function_defs: Vec<FunctionDefinition>,
}
```

1) `file_format::StructDefinition`:
```
pub struct StructDefinition {
	/// The `StructHandle` for this `StructDefinition`. This has the name and the abilities
	/// for the type.
	pub struct_handle: StructHandleIndex,
	
	/// Contains either
	/// - Information indicating the struct is native and has no accessible fields
	/// - Information indicating the number of fields and the start `FieldDefinition`s
	pub field_information: StructFieldInformation,
}
```

It seems that we can derive `expansion::ast::StructName` and `expansion::ast::StructDefinition`.

2)`file_format::FunctionDefinition`:
```
pub struct FunctionDefinition {
/// The prototype of the function (module, name, signature).
pub function: FunctionHandleIndex,

/// The visibility of this function.
pub visibility: Visibility,

/// Marker if the function is intended as an entry function. That is
pub is_entry: bool,

/// List of locally defined types (declared in this module) with the `Key` ability
/// that the procedure might access, either through: BorrowGlobal, MoveFrom, or transitively
/// through another procedure
/// This list of acquires grants the borrow checker the ability to statically verify the safety
/// of references into global storage
///
/// Not in the signature as it is not needed outside of the declaring module
///
/// Note, there is no SignatureIndex with each struct definition index, so all instantiations of
/// that type are considered as being acquired
pub acquires_global_resources: Vec<StructDefinitionIndex>,

/// Code for this function.

#[cfg_attr(

any(test, feature = "fuzzing"),

proptest(strategy = "any_with::<CodeUnit>(params).prop_map(Some)")

)]

pub code: Option<CodeUnit>,

}
```

It seems that we can derive `expansion::ast::FunctionName` and `expansion::ast::Function`

```
pub struct Function {
	pub attributes: Attributes,        // pass
	pub loc: Loc,                      // check
	pub visibility: Visibility,        // check
	pub entry: Option<Loc>,            // check
	pub signature: FunctionSignature,  // check
	pub acquires: Vec<ModuleAccess>,   // check
	pub body: FunctionBody,
	pub specs: BTreeMap<SpecId, SpecBlock>, // taken care of
}
```

> **A good first order of business might be writing derivers: functions that take fields of `CompiledModules` and produce the fields of `expansion::ast::ModuleDefinition` we need.**
> 

3) `file_format::ConstantPool`
```
pub type ConstantPool = Vec<Constant>;

pub struct Constant {
	pub type_: SignatureToken,
	pub data: Vec<u8>,
}
```