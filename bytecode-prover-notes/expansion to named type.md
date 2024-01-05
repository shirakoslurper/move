`E::Type_` to `N::Type_` in `naming::translate::type_()`:
- `ET::Unit`
	- `NT::Unit`
- `ET::Multiple(Vec<Type>)`
	- Maps `type_()` onto every member of the `Vec<Spanned<E::Type_>>` wrapped by `ET::Multiple`
	- `NT::Unit` if length is zero
	- The unwrapped type (we unwrapped `Type` to get `Type_` by mapping `type_*(`) if length is 1
	- `NT::Apply` if length is n
- `ET::Ref(bool, Box<Type>)`
	- `NT::Ref`
	- `ET::Ref` and `NT:Ref` are very similar in that they are both defined as `Ref(bool, Box<Type>)`
	- The `bool` represents the mutability of the reference.
- `ET::UnresolvedError`
	- `NT::UnresolvedError`
- `ET::Apply(ModuleAccess, Vec<Type>)` (holds a vector of types) where the `expansion::ModuleAccess` variant is `ModuleAccess::Name`
	- `NT::UnresolvedError` if we fail to resolve the unscoped type given context
	- `NT::builtin_` if we resolve the unscoped type to `RT::BuiltInType`
	- If we resolve the unscoped type to `RT::TParam`
		- `NT::UnresolvedError` if the vector of types held by `ET::Apply` is empty
		- `NT::Param` if the vector of types held by `ET::Apply` is not empty
- `ET::Apply(ModuleAccess, Vec<Type>)` (holds a vector of types) where the `expansion::ModuleAccess` variant is `ModuleAccess::ModuleAccess`
	- `NT::UnresolvedType` if we fail to resolve the module type given the module identity and name in `ModuleAccess(ModuleIdent, Name)`
	- `NT::Apply` if we resolve the module type
- `ET::Fun(Vec<Type>, Box<Type>)`
	- panics since this is only for spec use
	- these don't make it past the expansion stage so they shouldn't exist in the compiled bytecode

**Structs to know:**
`ModuleAccess(ModuleIdent, Name)`:
- `Name` is from `move_ir_types::location`. We've seen it before.
- `ModuleIdent_` consists of and `Address` and `ModuleName`
```
pub struct ModuleIdent_ {
	pub address: Address,
	pub module: ModuleName,
}

pub type ModuleIdent = Spanned<ModuleIdent_>;
```

This shouldn't be difficult to derive from a `CompiledModule`.

Otherwise, `E::Type_` just contains `bool` or `Type`.

### The significance of `context.resolve_unscoped_type(...)`, `context.resolve_module_type(...)`, and `N::BuiltinTypeName_::resolve(...)`

This is where we begin associating `expansion::ast::Type` with the variants of `SignatureToken`.

We call `context.resolved_unscoped_type(..)` when we're dealing with an `ET::Apply(ModuleAccess, Vec<Type>)` where the `ModuleAccess` variant is `Name(Name)`. This unscoped type resolution can resolve to three things: `None`, `Some(RT::Builtintype)`, and `Some(RT::TParam(_,tp)_))`. In the case of `Some(RT::BuiltinType)`, we call `N::BuiltinTypeName_::resolve(...)`.

We resolve builtin type name by taking a `&str` and returning a variant of `N::BuiltinTypeName_`. It's simple matching and should be easy to reverse.

When we call this in `naming::translate::type_()` it looks a little like this:
```
fn type_(context: &mut Context, sp!(loc, ety_): E::Type) -> N::Type {
	use ResolvedType as RT;
	use E::{ModuleAccess_ as EN, Type_ as ET};
	use N::{TypeName_ as NN, Type_ as NT};
	let ty_ = match ety_ {
		... => ...,
		ET::Apply(sp!(_, EN::Name(n)), tys) => match context.resolve_unscoped_type(&n) {
			None => {
				assert!(context.env.has_errors());
				NT::UnresolvedError
			}
			Some(RT::BuiltinType) => {
				let bn_ = N::BuiltinTypeName_::resolve(&n.value).unwrap();
				let name_f = || format!("{}", &bn_);
				let arity = bn_.tparam_constraints(loc).len();
				let tys = types(context, tys);
				let tys = check_type_argument_arity(context, loc, name_f, tys, arity);
				NT::builtin_(sp(loc, bn_), tys)
			}
			... => ...,
		}
		... => ...,
	}
}
```

It seems that the builtin type name is stored as a `String` or `&str` in the `ModuleAccess` component of `ET::Apply`.

> Focus on `N::BuiltinTypeName_::resolve(&n.value).unwrap()
> `&n` is a reference to `Name` which is a `Spanned<Symbol>`.
> 
> The `value` field of `Spanned<T>` returns `T`. The other field of `Spanned` is `loc`.
> 
> So, `&n.value` should return a `Symbol`.
> Ahhh, according to `move_symbol_types::symbol`, we implement dereferencing `&Symbol` to `&str`.
> We've also implemented the traits `From<&str>` and `From<String>`. This should be useful.
> 
> 
> `Symbol`
> `Symbol` is in `move_ir_types::location`.

Since we know that builtin types can be resolved to `ET::Apply(...)` we've taken care of:
- `U8`
- `U16`
- `U32`
- `U64`
- `U128`
- `U256`
- `Address`
- `Bool`
- `Vector`
- `Signer`

> I'm assuming that `TParam` means type parameter or generic.

> "Note that `(e)` does not have type `(e): (t)`, in other words there is no tuple with one element. If there is only a single element inside of the parentheses, the parentheses are only used for disambiguation and do not carry any other special meaning."

The above explains why in `parser::syntax::parse_type(..)` we just use the inner type of the 'tuple' of types of length 1 as the type.

Right now, we want to find out the fields of the type `bool`.

After looking through  `parser::syntax::parse_type(..)`, it does seem that anything that is not a unit (), reference &a, mutable reference &mut, or function ||, is of type Apply.

> **Important note about Multiple**
> Multiple is only used for return values and expression blocks.