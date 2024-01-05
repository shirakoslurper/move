The expanded module here (taken by our call `module_translator.translate(...)`) is likely the expansion for the module we want to assess.

More specifically we need `expansion::ast::ModuleDefinition` for:
- `decl_ana()`
- `def_ana()`
- `collect_spec_block_infos()`
- `translate_attributes()`

`function_infos` is only needed for `def_ana()`

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