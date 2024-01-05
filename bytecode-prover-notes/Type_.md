We need to get from `file_format::SignatureToken` to `Spanned<expansion::ast::Type_>`.

There isn't a one-to-one mapping of `SignatureToken` variants to `E::Type_` variants.

There is however a near one-to-one mapping of variants for:
- `P::Type`
- `E::Type`
but the parsing step is irrelevant.

We'll likely find transforms in later steps of compilation (after expansion).

`E::Type_`:
```
pub enum Type_ {
	Unit,
	Multiple(Vec<Type>),
	Apply(ModuleAccess, Vec<Type>),
	Ref(bool, Box<Type>),
	Fun(Vec<Type>, Box<Type>),
	UnresolvedError,
}
```

`file_format::SignatureToken`:
```
pub enum SignatureToken {
	/// Boolean, `true` or `false`.
	Bool,
	/// Unsigned integers, 8 bits length.
	U8,
	/// Unsigned integers, 64 bits length.
	U64,
	/// Unsigned integers, 128 bits length.
	U128,
	/// Address, a 16 bytes immutable type.
	Address,
	/// Signer, a 16 bytes immutable type representing the capability to publish at an address
	Signer,
	/// Vector
	Vector(Box<SignatureToken>),
	/// User defined type
	Struct(StructHandleIndex),
	StructInstantiation(StructHandleIndex, Vec<SignatureToken>),
	/// Reference to a type.
	Reference(Box<SignatureToken>),
	/// Mutable reference to a type.
	MutableReference(Box<SignatureToken>),
	/// Type parameter.
	TypeParameter(TypeParameterIndex),
	/// Unsigned integers, 16 bits length.
	U16,
	/// Unsigned integers, 32 bits length.
	U32,
	/// Unsigned integers, 256 bits length.
	U256,
}
```

Though there are a few related elements (mostly pertaining to references) these two definitions differ massively in terms of content. `SignatureToken` has a much higher level of specificity and is devoid of function, unit, multiple, and apply related elements.

> **Potentially important:**
> We can return the `AbilitySet` of a `SignatureToken` give n a context with `binary_view::abilities`.

In `ir_to_bytecode::compile_fields` we get a `SignatureToken` by calling the subroutine `compile_type(...)`.

`ir_to_byytecode::compile_type(...)`
```
fn compile_type(
	context: &mut Context,
	type_parameters: &HashMap<TypeVar_, TypeParameterIndex>	
	ty: &Type,
) -> Result<SignatureToken> {...}
```

`compile_type(...)` matches `move_ir_types::ast::Type` variants to their corresponding `SignatureTokens`. They are one-to-one.

As evidenced in `compiler::to_bytecode::translate()`, we go from `hlir`'s type structs/enums to `IR::Type`. This function calls `single_type(..)` to convert between the HLIR and IR types.

```
fn single_type(context: &mut Context, sp!(_, st_): H::SingleType) -> IR::Type {
	use H::SingleType_ as S;
	use IR::Type as IRT;
	
	match st_ {
		S::Base(bt) => base_type(context, bt),
		S::Ref(mut_, bt) => IRT::Reference(mut_, Box::new(base_type(context, bt))),
	}
}
```

More general type conversions are done with `types()`:
```
fn types(context: &mut Context, sp!(_, t_): H::Type) -> Vec<IR::Type> {
	use H::Type_ as T;	
	match t_ {
		T::Unit => vec![],
		T::Single(st) => vec![single_type(context, st)],
		T::Multiple(ss) => ss.into_iter().map(|st| single_type(context, st)).collect(),
	}
}
```

> FYI, HLIR means high level intermediate representation.
> CFGIR likely means control flow graph intermediate representation.

`hlir::ast::Type`:
```
pub enum Type_ {
	Unit,
	Single(SingleType),
	Multiple(Vec<SingleType>),
}

pub type Type = Spanned<Type_>;

```
Here, `Single` and `Multiple` wrap `SingleType`.

`SingleType`:
```
pub enum SingleType_ {
	Base(BaseType),
	Ref(bool, BaseType),
}

pub type SingleType = Spanned<SingleType_>;

```

`SingleType` implements functions that builds a `SingleType(BaseType)` from a `Loc`. 

> Between `Type_`'s variants and `SingleType`'s variants we have direct mappings to `Unit`, `Multiple`, and `Ref` from `E::Type_`.

`BaseType`
```
pub enum BaseType_ {
	Param(TParam),
	Apply(AbilitySet, TypeName, Vec<BaseType>),
	Unreachable,
	UnresolvedError,
}

pub type BaseType = Spanned<BaseType_>;
```

> Now we also have direct mappings to `Apply` and even `UnresolvedError` (though I suspect that `UnresolvedError` is and artifact of expansing parser AST and thus not relevant to us working backwards from a compiled module).

An example of a function creating a `SingleType`:
```
impl SingleType_ {
	pub fn base(sp!(loc, b_): BaseType) -> SingleType {
		sp(loc, SingleType_::Base(sp(loc, b_)),
	}

	pub fn bool(loc: Loc) -> SingleType {
		Self::base(BaseType_::bool(loc))
	}
	...
}
```

`BaseType_` implements functions such as the `Bool(...)` function we see above.

`impl BaseType`:
```
impl BaseType_ {

	pub fn builtin(loc: Loc, b_: BuiltinTypeName_, ty_args: Vec<BaseType>) -> BaseType {
		use BuiltinTypeName_::*;
		
		let kind = match b_ {
			U8 | U16 | U32 | U64 | U128 | U256 | Bool | Address => AbilitySet::primitives(loc),
			Signer => AbilitySet::signer(loc),
			Vector => {
				let declared_abilities = AbilitySet::collection(loc);
				let ty_arg_abilities = {
					assert!(ty_args.len() == 1);
					ty_args[0].value.abilities(ty_args[0].loc)
				};
				AbilitySet::from_abilities(
					declared_abilities
						.into_iter()
						.filter(|ab| ty_arg_abilities.has_ability_(ab.value.requires())),
				)
				.unwrap()
			}
		};
		let n = sp(loc, TypeName_::Builtin(sp(loc, b_)));
		sp(loc, BaseType_::Apply(kind, n, ty_args))
	}
	
	pub fn abilities(&self, loc: Loc) -> AbilitySet {
		match self {
			BaseType_::Apply(abilities, _, _) | BaseType_::Param(TParam { abilities, .. }) => {
				abilities.clone()
			}
			BaseType_::Unreachable | BaseType_::UnresolvedError => AbilitySet::all(loc),
		}
	}
	
	pub fn bool(loc: Loc) -> BaseType {		
		Self::builtin(loc, BuiltinTypeName_::Bool, vec![])
	}
	
	pub fn address(loc: Loc) -> BaseType {
		Self::builtin(loc, BuiltinTypeName_::Address, vec![])
	}
	
	...
}
```

> My question is, how do we know to call `bool()` or `address()` or `u8()`?

Given that `BaseType_::Param` isn't ever created in the implemented functions for `BaseType` I'm assuming that is must be created elsewhere. The creators of that variant and the callees of `SingleType_` functions (which linearly call `BaseType_` functions) liekely hold an answer. 

> We can probably find these callees in `hlir::translate`.
> Also, I htink the `sp(...)` function we've been seeing everywhere is used to create a `Spanned<T>`.

In `move_ir_types::location`, we find proof that `sp()` is used to created `Spanned`s.
```
/// Function used to have nearly tuple-like syntax for creating a Spanned
pub const fn sp<T>(loc: Loc, value: T) -> Spanned<T> {
	Spanned { loc, value }
}
```

In `hlir::translate`, the function in charge of converting an iterator of `BaseType`s is `base_types(...)`, which calls `base_type(...)`.
```
fn base_types<R: std::iter::FromIterator<H::BaseType>>(
	context: &Context,
	tys: impl IntoIterator<Item = N::Type>,
) -> R {
	tys.into_iter().map(|t| base_type(context, t)).collect()
}
```

The function signature of `base_type(..)`:
```
fn base_type(context: &Context, sp!(loc, nb_): N::Type) -> H::BaseType {...}
```

`N` in this case is `naming::ast`, likely a step between the expansion and HLIR stages of compilation.

`N::Type`:
```
pub enum Type_ {
	Unit,
	Ref(bool, Box<Type>),
	Param(TParam),
	Apply(Option<AbilitySet>, TypeName, Vec<Type>),
	Var(TVar),
	Anything,
	UnresolvedError,
}

pub type Type = Spanned<Type_>;
```

`E::Type_` again:
```
pub enum Type_ {
	Unit,
	Multiple(Vec<Type>),
	Apply(ModuleAccess, Vec<Type>),
	Ref(bool, Box<Type>),
	Fun(Vec<Type>, Box<Type>),
	UnresolvedError,
}
```

Still not quite one-to-one. However, naming *is* the step between expansion and HLIR.

In fact, the function `type_()` in `naming::translate` tranlates `E::Type_` to `N::Type_`
