Now, *at the moment*, this is only with regards to creating `E::Type`. 

My question now is what non-struct types hold in the `Vec<Type>` portion of `E::Type_::Apply(ModuleAccess::Name(Name), Vec<Type>)`.

Is size 0 ok? I'm assuming it is unless we keep going down infinitely. 
> By the above I mean that if every `Apply` was required to hold a `Vec<Type>` with length at least 1 then the depth of the type would be infinite.

There are two variants of `TypeName_`: `Builtin(..)` and `ModuleType(..)`.

**Implementation for builtin types**

`type_from(signature_token: &F::SignatureToken, compiled_module: &F::CompiledModule) -> E::Type {..}` matches the `&F::SignatureToken` variants and outputs a `E::Type` (a `Spanned<E::Type_>`).

Builtin types are simply instantiated as `ET::Apply(ModuleAcccess, Name)` with `ModuleAccess_(Name(Name))`.

The `ModuleAccess` variants:
```
pub enum ModuleAccess_ {
	Name(Name),
	ModuleAccess(ModuleIdent, Name),
}
```

> `location::Name` and `E::ModuleAccess`.

> Builtin types have `ModuleAccess::Name(..)` and they don't need to refer to the modules in wich they are defined. The `Name` is simply built from the string associated with the builtin type. So and `"int"` `&str` for a Move `int`.

**Implementation for structs**
Structs are handled a little differently. 

We're still sticking with `ET::Apply(..,..)` as it's the only one used in Struct fields.
But we're using a differnet vriant of `ModuleAccess`, the `ModuleAccess(ModuleIdent, Name)` variant.

`E::ModuleIdent` is a `Spanned<E::ModuleIdent_>`.
`E::ModuleIdent_`:
```
pub struct ModuleIdent_ {
	pub address: Address,
	pub module: P::ModuleName,
}
```

`P::ModuleName` can be instantiate with `P::ModuleName(lcoation::Identifier)`.

So we need:
- The module's `location::Identifier`
- The module's `Address`

> These are more closely associated with `ModuleType` variant of `hlir::ast::TypeName_`. 

The `TokenSignature::Struct(Box<StructHandleIndex>)` provides us with a pointer to the `StructHandleIndex`. Using this index, we can find the associated `StructHandle` given `&CompiledModule` with `<CompiledModule>.struct_handle_at(<StructHandleIndex>)`.

`StructHandle`:
```
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
```

>  We can get the `Name` field of `ModuleAccess::ModuleAccess` with `<CompiledModule>.identifier_at(<StructHandle>.module)` and instantiating that result with `P::ModuleName(<Identifier>)`.

We can get the `ModuleHandle` for our struct's origin module with `<CompiledModule>.module_handle_at(<StructHandle>.module).`

`ModuleHandle`:
```
pub struct ModuleHandle {
	/// Index into the `AddressIdentifierIndex`. Identifies module-holding account's address.
	pub address: AddressIdentifierIndex,
	
	/// The name of the module published in the code section for the account in `address`.
	pub name: IdentifierIndex,
}
```

`E::Address` has two variants:
```
pub enum Address {
	Numerical(Option<Name>, Spanned<NumericalAddress>),
	NamedUnassigned(Name),
}
```

I'm assuming that any CompiledModule is going to have an explicitly defined `AccountAddress`, placing everything under the `Numerical` variant.

`NumericalAddress` is `move_command_line_common::address::NumericalAddress`:
```
pub struct NumericalAddress {
	/// the number for the address
	bytes: AccountAddress,
	
	/// The format (e.g. decimal or hex) for displaying the number
	format: NumberFormat,
}
```

> `AccountAddress` is `NumericalAddress` is `move_core_types::account_address::AccountAddress`.
> `NumberFormat` is `command_line_common::parser::NumberFormat`.

`NumberFormat`:
```
pub enum NumberFormat {
	Decimal = 10,
	Hex = 16,
}
```

Instantiating looks a bit like:
```
pub const fn new(bytes: [u8; AccountAddress::LENGTH], format: NumberFormat) -> Self {
	Self {
		bytes: AccountAddress::new(bytes),
		format,
	}
}
```

> From looking through `tools::move_package::compilation::compiled_package`, it seems like the default is `Hex`. I'm getting that `NumberFormat` is purely aesthetic especially since we are holding the actual array of bytes in `AccountAddress`.

`move_core_types::Identifier`:
```
pub struct Identifier(Box<str>);
```

>  `Box<str>` really is just `Box<[u8]>`. Unlike a string, it is not resizable. Unlike `&[T]` and `& mut[T]`, `Box<[T]>` owns its data.
> 
> "`String` stores pointer+length+capacity while `Box<str>` stores pointer+size. The capacity allows `String` to append efficiently. The compiler uses `Box<str>` as an optimization when it has a massive number of immutable strings so size matters, for example the string interner:"
> 
> `<Identifier>.into_string` works on the value not a reference
> We can't use `borrow()` directly since `Borrow` is a private trait that is not exported from module `move_core_types::identifier`. We can, however, use `<&Identifier>.as_ident_str()` which gives us the borrowed `IdentStr`.

> A trait must be in scope to call it's methods.

Given that an `Identifier` is just a tuple struct wrapping a `Box<str>` (not a `Box<&str>`!), we might be able to make a `Name` from the `Identifier` itself.

We know a `Name` is a `Spanned<Symbol>` and that a `Symbol`. `Symbol` has the `From` trait implemented for a couple type. Let's test `Identifier`.

### The issue with the `Option<Name>` field of `E::Address`:
The address of our module isn't necessarily goin to have a name. And it shouldn't need one!

### Vector of Types
Does Struct version of `Type_:Apply` hold a vector of the types it holds a.k.a. the types of its fields?

Judging from [`typing::translate::visit_type_params()`](https://github.com/move-language/move/blob/main/language/move-compiler/src/typing/translate.rs), `N::Type_::Apply(..,.., ty_args)` with a *non-empty* vector of `ty_args` can hold both a `N::TypeName_::Builtin(..)` and an `N::TypeName_::ModuleType()` as type names.

## parser ModuleName

It is instantiated with a Name.