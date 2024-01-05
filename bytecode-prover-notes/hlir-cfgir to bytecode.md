### hlir BaseType instantiation
`H::BaseType_` implements functions that allow us to specify the abilities and type names of our given types.


### hlir to cfgir

We don't seem to translate from an HLIR-specific type definition to a CFGIR-specific type definition.

It seems that the structs representing function bodies and contants simply use the `hlir::BaseType` and `hlir::SingleType` enums.

### cfgir to bytecode

