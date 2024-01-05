The 'target' struct is given by `EA::SpecBlockTarget` wher `EA = expansion::ast`.

> Can this and it's members be derived from `CompiledModule`?

With regards to `SpecBlockTarget`s, `ModuleBuilder` functions only seem concerned with the `value` and `loc` fields.

> We don;t have much to deal with if `value` is `SpecBlockTarget_::Module` as opposed to `SpecBlockTarget_::Member(_)` or `SpecBlockTarget_::Schema(__)`.
