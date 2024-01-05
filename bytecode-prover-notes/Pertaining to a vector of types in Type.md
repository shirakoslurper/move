[`typing::core`](https://github.com/move-language/move/blob/main/language/move-compiler/src/typing/core.rs) seems to be what we're looking for.

`make_struct_type()`:
```
pub fn make_struct_type(
	context: &mut Context,
	loc: Loc
	m: &ModuleIdent,
	n: &StructName,
	ty_args_opt: Option<Vec<Type>>,
) -> (Type, Vec<Type>) {...}
```

Seems like we can take a vector of types. What do they become though? Fields?

We can look at where it is called in `translate`. It's called in the `exp_inner()` function after the expression is patched with `NE::Pack`. 

The `Exp_::Pack` variant:
```
Pack(ModuleIdent, StructName, Option<Vec<Type>>, Fields<Exp>),
```

Judging from the implementation, it seems like the vector of type arguments created with `make_struct_type()` is used to type the fields with a call to `add_field_types()`. Seems that field types are constructed from the type arguments.

**`add_field_types()`:**
we make potential field type given the type arguments with `core::make_field_types()`, but **not** the fields themselves (of enum `StructField`).
- We grab the struct definition from context with the `MouduleIdent` and `StructName`
- We create substitute type parameters, passing type arguments
- We match the fields of our struct definition with correct variant
	- Native: we just return the *loc*
	- Defined: we take `Fields<Type>` and substitute type params with what we made earlier and our field types held in `Fields<Type>`
	- `make_tparam_subst()` returns `TParamSubst`
		- **This is what takes the type arguments. Try to understand it.**
		- We take type parameters and type arguments
		- We make sure they are of equal length
		- We instantiate and empty `TParamSubst`
		- We iterate through both (parameters and arguments) zipped together
			- **The zipping insinuates a sort of ordering.**
			- We insert into the subtitute struct
				- The type parameter id as the key
				- And the argument and the value
	- `subst_tparams()` returns a `Type`!
		- we pass a `TParamSubst` and `Type`
		- we match the `N::Type_` cases
		- Recursive and our base case is that we match the `Param` variant
			- We return the type argument from `make_tparams_subst()`

Simpler:
- create mapping of type parameters to type arguments
- 

```
pub type TParamSubst = HashMap<TParamID, Type>;
```

```
pub struct TParam {
	pub id: TParamID,
	pub user_specified_name: Name,
	pub abilities: AbilitySet,
}
```

> It's interesting that we start with `N::StructField`s and then we end with typed `N::StructField`s.

> Why `TParamSubst`? Understand why?
> Question: Why do we need `TParamSubst`?
> We get a mapping of the type parameter id to type. We pass this to every call to `subst_tparams`
> Question: How does `TParamSubst` help us make the `Type` for the field?
> If we're already at the base case of the `N::Type_` variant being `Param` then we just return the type associated with that type parameter.
> 
> So does every type have a type parameter?
> 
> What is the relationship between type parameters and type arguments? Seems one-to-one since they are zipped together?
> How is ordering determined?
> 
> 3 way relationship: `StructField`, `StructTypeParameter`, `Type`

```
pub enum StructFields {
	Defined(Fields<Type>),
	Native(Loc),
}
```

> How do we know what type argument to match with what field?

We then match the variants - only `Defined` is valid.
We grab the `E::Fields<Type>` stored inside which is a `UniquMap`:
```
pub type Fields<T> = UniqueMap<Field, (usize, T)>;
```
> Perhaps thats why they use the variable name `m` before `fields_ty`.

We then iterate through the map and get the `Field` only and ensure that the fields we want to make field types from are present in the products of `make_field_types`.

We then map the fields we originally had:
- We remove the field type from field types with our field key.

> We need to derive type arguments and type paramerters.
> Actually type parameters im not too sure about...
> Like are they simply separated from the rest of the type args. Or do they have to be derived.
> Like are they present as their own unlabeled entities in the expansion ast?
> 
> In Naming, we get type parameters, `TParams`, from `StructDefinition`'s to extract the abilities of those type parameters and pass them as **constraints**
> 
> Hmm so its funky. We ge the structs type definition, then its type params, then the abilities of those type params. We then apply the abilitie sas contraints. Then into a type variabels vector we pass the product of making type params (likely different ast) with the `TVarCase` (which will always be `Base`) and then we will make the type params given the context, location, case, and contraints for that location. 
> 
> **`instantiate_type_args()`** calls instantiate so there some recurisve stuff going on in terms of recursive type instsntiation.

There's one case where no vector of types is given, BUT we do add type parameters.

There's another case where a vector of types is given and we call `instantiate_apply()` on the vector. 

Now, `instantiate_apply()`'s inner working involve matching the `TypeName` that is passed to it.

> This means `make_struct_type()` knows the type name. In fact, since it is making a struct type it sets the `TypeName` to module type, the only type it can be.

**Also, I learned that non Primitive types should NOT have "abilities". Their ability sets are instead derive from the abilities of their type parameters.**

**`make_struct_type` steps (with multiple type arguments):**
- create a `TypeName` as `ModuleType` (type defined in a module)
- call `instantiate_apply()` with the `TypeName` and type arguments
- match with `ModuleType` and get the struct type parameters (`context.struct_tparams()`) given
	- context
	- `ModuleIdent` (REFER RIGHT)
	- `StructName` (REFER RIGHt)
- call `instantiate_type_args()` with the type arguments we had and our new type constraints (type parameter abilities)
- if the call succesfully instantiates and apply, clone the type arguments 

