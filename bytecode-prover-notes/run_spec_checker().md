Since we have a spec, we will need to populate our environment similar to how we populate our environment with `run_spec_checker()`.

This function, however, requires a couple elements from `AnnotatedCompiledUnit`.:
- `CompiledModule` (provided)
- `SourceMap` (provided)
- `FunctionInfo`s (derivable?)

**It also requires an `expansion::ast::Program`.**

> Can we derive an `expansion::ast::Program`? One that properly holds the specs it is meant to?

```
pub struct Program {
	// Map of declared named addresses, and their values if specified
	pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
	
	pub scripts: BTreeMap<Symbol, Script>,
}
```

And what about `expansion::ast::ModuleDefinition` (in `expansion::ast::Program`)?
```
pub struct ModuleDefinition {

	// package name metadata from compiler arguments, not used for any language rules
	pub package_name: Option<Symbol>,
	
	pub attributes: Attributes,
	
	pub loc: Loc,
	
	pub is_source_module: bool,
	
	/// `dependency_order` is the topological order/rank in the dependency graph.
	/// `dependency_order` is initialized at `0` and set in the uses pass
	pub dependency_order: usize,
	
	pub immediate_neighbors: UniqueMap<ModuleIdent, Neighbor>,
	
	pub used_addresses: BTreeSet<Address>,
	
	pub friends: UniqueMap<ModuleIdent, Friend>,
	
	pub structs: UniqueMap<StructName, StructDefinition>,
	
	pub functions: UniqueMap<FunctionName, Function>,
	
	pub constants: UniqueMap<ConstantName, Constant>,
	
	pub specs: Vec<SpecBlock>,
}
```

We can derive `source_module` and `dependency_order` from `CompiledModule`.

> We do this ordering derivation (implicitly) before calling `run_bytecode_model_builder()`. This function takes a topologically sorted vector of `CompiledModule`s.

In `read_write_set::analyze()`, which contains a call to `run_bytecode_model_builder()` we topologically order our unordered vector of `CompiledModule`s:
```
pub fn analyze<'a>(
	modules: impl IntoIterator<Item = &'a CompiledModule>,
) -> Result<ReadWriteSetAnalysis> {
	let module_map = Modules::new(modules);
	let dep_graph = module_map.compute_dependency_graph();
	let topo_order = dep_graph.compute_topological_order()?;
	analyze_sorted(topo_order)
}
```

We can derive structs from `struct_defs` in `CompiledModule` and `functions` from `function_defs` in `CompiledModule`.

Dunno about contants...

As for specs it seems best to parse and expand.

> Does expanding specs require other information about the parsed module besides the specs?

> What is `Context` exactly?

**Used elements of `expansion::ast::ModuleDefinition` in `ModuleBuilder` functions:**
- `attributes: Attributes`
- `structs: UniqueMap<StructName, StructDefinition>`
- `functions: UniqueMap<FunctionName, Function>`
- `constants: UniqueMap<ConstantName, Constant>`
- `specs: Vec<SpecBlock>`

**Unused elements of `expansion::ast::ModuleDefinition` in `ModuleBuilder` functions:**
- `package_name: Option<Symbol>`
- `loc: Loc`
- `is_source_module: Bool`
- `dependency_order: usize`
- `immediate_neighbors: UniqueMap<ModuleIdent, Neighbor>`
- `used_addresses: BTreeSet<Address>`
- `friends`