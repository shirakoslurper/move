One of the `ModuleMember` variants is `Spec`

> Refer to `module_(...)` in `expansion::translate`.  This function calls `spec()` that, with `Context`, turna a `parser::ast::SpecBlock` into a `expansion::ast::SpecBlock`.

What elements of `Context` are used in constructing a `Spec`?
- We flatten attributes according to context and the spec's attributes.
- We turn `in_spec_context` on and off before and after creating a new scope.
- Calles `uses(...)` which seems to create a new scope and call `use_(...)` for each use. This takes use declarations and 
	- This processes `use <Module/Members>` declarations and adds module aliases for either ths whole module or members of it.
	- `use_()` checks that a module is bound (for both modules and members) - that source code for the used module exists. It then adds aliases for the module or members using an `mut AliasMapBuilder`.
	- 
- We call `add_and_shadow_all(...)` on our existing `AliasMap` in `Context`. This means it adds all of the new items in the new inner scope as shadowing the outer one and gives the outer one.
- We then convert parsed `SpecMember`s into expansion `SpecMembers`.
- We then set context to outer scope with our old aliases.
	- `set_context_to_outer_scope(...)` resets the alias map and reports errors for unused aliases
- `E::SpecTarget` is made by calling `spec_target(context, target)`. Context only matters when are target is either a `Schema` or a `Member`. Even then, the conversion can be done fairly easily, though we will have to be aware of the function signtures.

> The use of certain expressions is only allowed in spec context. It allows expressions in specs to be properly processed.

> What are spec attributes or attributes in general?
> Attributes are based on the Rust attribute syntax and can be attached to address blocks, modules, scripts, and any module top level member. They are currently used for unit testing.
> If these are the right attributes we've found in the docs, then we shouldn't be too concerned about them.

**Thoughts**
Given that the only infromation from `Context` specs really need (for expansion) are with regards to the `AliasMap` or the use modules and/or module members, it seems that expanding `SpecBlock` is bytecode agnostic.

**We don't need further stages of compilation beyond expansion, at least with regards to spec blocks.**