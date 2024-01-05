**WRT `file_format::StructDefinition`s**
The `struct_handle` field of `StructDefinition` provides us with a `StructHandleIndex` so that we can index into the `struct_handles: Vec<StructHandle>` field of `CompiledModule`.

So, if we want to access a `StructHandle` given a `StructDefinition`, we need to provide at least a reference to the origin `CompiledModule`. So, we might as well process the whole `CompiledModule`.

> Should we consume elements of the compiled module of not? For example, the `struct_defs:  Vec<StructDefinition>`?
> Only if we intend to pass a clone of `Compiled`

**Loc**
We do not now the `ByteIndex` into the source file since we are not given the source file. `run_bytecode_model_builder` just uses a `Loc::default()`. 

> **Constructing Struct Data**
> `run_bytecode_model_builder` shows how to create symbol. It works as long as we have the `CompiledModule` and an `IdentifierIndex`.

**`AbilitySet`**
There is a one-to-one struct called `AbilitySet` in both `file_format` and `expansion::ast`.

In `file_format`, `AbilitySet` is a tuple struct wrapping a `u8`.

**`TypeParameters`**
There is a one-to-one struct in `file_format` and `expansion::ast`.

`file_format::StructTypeParameter`:
```

```

`expansion::ast::StructTypeParameter`:
```
pub struct StructTypeParameter {
	/// The type parameter constraints.
	pub constraints: AbilitySet,
	/// Whether the parameter is declared as phantom.
	pub is_phantom: bool,
}
```

> **`Name`**
> `Name` is from `move_compiler::shared` and it is a `Spanned<Symbol>`.

**Type Parameter**
Type parameters are generics and vice versa. Name isn't significant here.

Should we make changes to a mutable `expansion::ast::ModuleDefinition`? Or just return things piecewise?