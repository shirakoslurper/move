`naming::translate` and `hlir::translate` do not create `N::Type_`'s with known type names. So where does it happen?

> 	We assume that these type names are likely created somewhere because we know that `TypeName`s are present in `N::BaseType_`. 

### The typing step
HLIR isn't the step that comes immediately after Naming. We can tell by how the `program(..)` function takes a `T::Program`. We do however, preserve the use of `N::Type`. `typing::ast` does not have its own `Type` struct.

Instead `N::Type` is extensively used as a member of `typing::ast`'s many structs.

### The Naming ro HLIR Bridge
When defining `Type_` in the HLIr stage, the HLIR AST is aware of the Naming AST.


