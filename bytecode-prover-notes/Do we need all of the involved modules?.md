`move_model::run_bytecode_model_builder` - our inspiration a little bit - starts with an iterator of `CompiledModule`s.

### Why do we need multiple CompiledModules to build the model?

Actually this may be a misleading detour.

Let's look at the normal `run_model_builder...` again.

### `PackagePaths`
From `move_compiler::shared::PackagePaths`.
I believe this struct contains all the paths to the sources in the package we want compiled.

The compiler is built based off of these dependencies..

### What is the difference between a program and a module?

Why does a program contain multiple modules? I'm just assuming a program is a package...

Aahhhh.
Remember, running the prover will run it on all specified modules (in the package directory?).


