### expansion
Type parameters are not a `Type_` variant in expansion
There is a `StructTypeParameter` struct for `StructDefinition` purposes, however.

It does *not* seem like type paramters are additional type arguments of a type as a I originally thought. They simply influence how those type arguments are built/defined by way of constraints based on abilities and whatnot.