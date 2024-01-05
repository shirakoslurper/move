`run_spec_checker()` attempts to match all `AnnotatedCompiledUnit`'s module identities with a member of `expansion::ast::Program`.

This might meen that we need an `expansion::ast::Program` that contains all the modules besides our target one.

> I guess we can do that? Somewhat?

> **WAIT NEVERMIND**. We already considered this. We're rederining a minimal but functionally complete `E::Program`from a `CompiledModule`. We just need to merge specs into this `expansion::ast::Program` before we pass it to something like `run_spec_checker()`.

> **A big takeaway is that we NEED ModuleBuilder to do some work to transform our `SpecBlock`s into `Spec`s.