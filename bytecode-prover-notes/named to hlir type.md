`N::Type_` to `H::Type_`
- `NT::Unit`
	- `type_()`: `HT::Unit`
- `NT::Ref(bool, Box<Type>)`
	- `type_()`: `HT::Single(SingleType)`
		- `single_type()`: `H::SingleType_::Ref()`
			- `base_type()`: We should not reach this although we will be calling it for the value the reference refers to. We throw a contraint error
- `NT::Param(TParam)`
	- `type_()`: `HT::Single(SingleType)`
		- `single_type()`: `H::SingleType_::Base(BaseType)`
			- `base_type()`: `H::BaseType_::Param(N::TParam)`
- `NT::Apply(Option<AbilitySet>, TypeName, Vec<Type>)
	- We `panic!` f theres is no `AbilitySet`
	- `type_()``HT::Multiple(Vec<SingleType>)` if there is an `AbilitySet`
		- We then do recursive calls to `type_(..)`
		- Refer to all the other types to see how they end up.
- `NT::Var(TVar)`
	-  `type_()`: `HT::Single(SingleType)`
		- `single_type()`: `H::SingleType_::Base(BaseType)`
			- `base_type()`: We `panic!`, which makes sense since we do not ever create the `NT::Var` variant in `naming::translate`'s routines.
- `NT::Anything`
	-  `type_()`: `HT::Single(SingleType)`
		- `single_type()`: `H::SingleType_::Base(BaseType)`
			- `base_type()`: `H::BaseType_::Unreachable`
- `NT::UnresolvedError`
	- `type_()``HT::Single(SingleType)`
		- `single_type()`: `H::SingleType_::Base(BaseType)`
			- `base_type()`: `H::BaseType_::UnresolvedError`


**Notes:**
`H::Type_` has 3 variants:
- `Unit`
- `Single(SingleType)`
- `Multiple(Vec<SingleType>)`

`H::SingleType_` has 2 variants:
- `Base(BaseType)`
- `Ref(bool, BaseType)`

`BaseType_` has 4 variants:
- `Param(TParam)`
	- `TParam` is from `naming::ast`
- `Apply(AbilitySet, TypeName, Vec<BaseType_>)`
- `Unreachable`
- `UnresolvedError`

`TypeName` has two variants:
- `Builtin(N::BuiltinTypeName)`
	- These refer to the builtin types of Move
- `ModuleType(ModuleIdent, StructName)`

We end up calling the `single_type()` function in every case besides `ET::Unit`. It takes some context and a `Spanned<Loc,  N::Type_>`.

`single_type(..)` internally calls `base_type`

> It seems that the keyword `Base` refers to a value not a reference.

