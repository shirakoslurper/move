Has a `ModelBuilder` parent (given upon instantiation) which holds `GlobalEnv` and the metadata in it.

Upon calling `translate()`, it takes a loc, an expanded module, a compiled module, a source map, and a vector of function info. 

Among the `ModuleBuilder` functions, there do seem to be a couple `parent.env` reliant ones that are *not* getters.

This is to be expected, since we are trying to populate the env from the result.

> What we want to know is: what steps *require* information that is provided by compilation steps after expansion?

> `module_translator.translate()` is the last call before populateing the `GlobalEnv` with model level information.

> `builder.populate_env()` just adds the `ModelBuilder's` intrinsics to the `GlobalEnv`'s. 