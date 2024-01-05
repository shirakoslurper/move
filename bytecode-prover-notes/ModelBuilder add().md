`<ModelBuilder>::add()` is called once in the whole codebase.

It is a subroutine of  `<ModuleBuilder>.populate_env()`, which is a sub

It seems that we then might be able to directly apply a module level spec without running through `run_spec_checker()`, which eventually makes a change to `module_data`

> **HOWEVER, the task of translating `SpecBlock`s into `Spec`s probably isn't something we should be doing alone.**