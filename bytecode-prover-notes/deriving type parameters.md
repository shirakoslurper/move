> Are types in the expansion stage necessarily struct definitions?

The expansion AST should only contains struct/type definitions not instantiations (right?).

Types do, however, seems dependent on scope but it's only post-expansion where it seems to matter.

> It isn't til the `typing` stage that we start recursively "passing" values into the type parameters: recall the zipping of type parameters and the `Vec<Type>` in `Type_::Apply(_,_)`.

`E::Type_::Apply(_,_)` stills holds a `Vec<Type>`.

Would it be fair to say that what we explicitly refere to as types are only ever members of things? Part of a signature?
A struct/type definition then wouldn't be the type itself (?)

Seems like we get a bit of clarity rom `CompiledModule`:
```
// Signature
// A signature can be for a type (field, local) or for a function - return type: (arguments).
// They both go into the signature table so there is a marker that tags the signature.
// Signature usually don't carry a size and you have to read them to get to the end.
```
Also it seems that a `SignatureToken` is explicity a type/struct definitio:
```
/// A type definition. `SignatureToken` allows the definition of the set of known types and their
/// composition.
```

**Referece Points**
A field can be of a type that takes a tyep argument.

```
struct Some {
	x: Vec<i32>,
}
```

**Consulting the disassembler**
Considering the disassembler should at the very least print the types defined in the module it's disassembling adn that it prints the type arguments of field in its type definitions, we should be able to learn something.

```
struct Pool<phantom Ty0, phantom Ty1> has key {
	frozen: bool,
	timestamp: u64,
	fee_bps: u64,
	swap_events: EventHandle<SwapEvent>,
	add_liquidity_events: EventHandle<AddLiquidityEvent>,
	remove_liquidity_events: EventHandle<RemoveLiquidityEvent>,
	x_reserve: Coin<Ty0>,
	y_reserve: Coin<Ty1>,
	lp_mint: MintCapability<LP<Ty0, Ty1>>,
	lp_burn: BurnCapability<LP<Ty0, Ty1>>
}
```

The disassembler starts with a source mapping (in our case we sould use the bytecode source map with no real source).
This seems crucial to the operation of everything.

The functions of interest are `disassemble_struct_def(&self, struct_def_idx: StructDefinitionIndex)` and its subroutine used to fetch and format field information `disassemble_sig_tok(&self, sig_tok: SignatureToken, type_param_context: &[SourceName])`.

It seems that we have access to all the struct definitions used by the module since our disassembler has to filter for only the ones that it defines:
```
// The struct defs will filter out the structs that we print to only be the ones that are
// defined in the module in question.
```

`disasseble_struct_def(...)`:
- We grab the struct source map for mthe source mapping
- We fetch the field info (if present) as a `Option<Vec<(&IdentStr, &TypeSignature)>>`
- Recall that `TypeSignature` is a wrapper over `SignatureToken`
- We grab the type parameters (to the struct) with `Self::disassemble_struct_type_formals(...)`
- We iterate over field info and call `disassemble_sig_tok()` with the `SignatureToken` and the type parameters present in the struct source map.

`disassemble_sig_tok(...)`:
- builitn types are handled by just printing the type name
- structs are handled by printing the struct name
	- these do *not* have type arguments
- structs instantiations have more of a process
	- we extract the vector of type arguments, `Vec<SignatureToken>`
	- we iterate over those type arguments and call `disassemble_sig_tok(...)` on them with the same type parameter context
		- this call should be recursive until it reaches a base case of any variant of `SignatureToken` that isn't `StructInstantiation`, `Vector`, `Reference`, of `MutableReference`
	- we call `format_type_params(&instantiations)`
		- seems like reuse but semantically it should be `format_type_arguments(...)`
		- we are passing type arguments
		- it's not describing parameters.
	- we grab the name of struct with the `StructHandleIndex` given in the `StructInstantiation` variant
	- we print `struct_name<type arguments>`.

I'm gathering that using the bytecode source mapping api should make things a great deal easier.

> `SingatureToken` dsicriminates between `StructInstantiation` and `Struct`! And `disassemble_sog_tok(...)` *doesn't* handle that variant as an error case!

> A little more info on the `StructInstantiation` variant of `SignatureToken`: 
> It looks a little like this`StructInstantiation(StructHandleIndex, Vec<SignatureToken>)`, and I'm guessing the contents are the struct definition and type arguments to instantiate the struct with.

**If we only recursively substitute type parameters for the purpose of making field types in the typing stage, what information is our type related structs missing in the expansion stage?**

**`typing::core::make_field_types()`**
This is the function that resursively substitutes type parameters with type arguments.
It is called by `typing::translate::add_field_types()` which is in turn called by `exp_inner(...)` when the provided `naming::ast::Exp` variant matches `Pack` or `Unpack`.

> "Packing" is more intuitive as a concept when you think of what the reverse of "unpacking" achieves - initializing a struct and its fields.

Upon matching `Pack`, we get typed fields by calling `add_field_types`. Were our fields untyped before this?

I'm guessing they aren't entirely untyped given the following enum definition in `expansion::ast`:
```
pub enum StructFields {
	Defined(Fields<Type>),
	Native(Loc),
}
```

> So then, it seems that we really only have to worry about struct defintiions and the struct instantiations within those definitions (the latter for field typing purposes).

### An aside on clap
Im sure that it utilizes a bunch of macros to write functions that parse arguments as we want.

### Implementing a recursive type deriver
I think i can go almost one-to-one in terms of implementing the recursive type deriver.
However, since expansion doesn't account for the packing and unpacking of structs outside of struct definition or function signature contexts, we do not have to account for the `StructInstatiation` `SignatureToken` variant. We can just treat that as a skip case.

### Overall, you're doing a variation on disassembly so follow the disassembly model to start. And get familiar with the source mappings

**Phantom type parameters**
In expansion, the phantomness of a `StructTypeParameter` is stored in the `StructTypeParameter` structs in the `type_parameters` field of `StructDefinition`.

> Know the difference between associated function syntax and methods. Associated functions don't act on the object. Methods work with an instance of the object.

**Type Parameter Field**
A field can also hold the type parameter.
```
struct Example<T> {
	some_field: T
}
```
How do we express a struct parameter "type" in the expansion struct?

I think we solve it with the `ModuleAccess::Name(Name)` variant. We can check with one of the later compilation steps when we extract the string from within the 

### ResolvedType
in the naming stage we have an enum `ResolvedType`
```
enum ResolvedType {
	TParam(Loc, N::TParam),
	BuiltinType,
}
```

This relates to the whole things about unscoped types that we have in our orange notebook.
`resolved_unscoped_type()` is a relavant function but what should be more relevant is how and when we decide to add the "type" to unscoped types.

OK so in `type_` in naming:
If we match apply with `ET::Apply(ModuleAccess::Name(name), __)` we use the `name` to resolve the unscoped type!!

So we just need to create a `ModuleAccess::Name(name)` with our type parameter name and provide an empty vector of types - else we throw a error in naming.

function calls:
- `type_parameter`
- `bind_type`