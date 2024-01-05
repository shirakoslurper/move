We're trying to provide the `E::Type` field of `E::StructField`.

> Hm. I assume that a struct field wouldn't contain the `Unit` or `Fun`. There's a chance it contains a reference but let's check. I would assume not because Move doesn't have lifetimes and introducing references would be dangerous without the inclusion of lifetimes.

> "Structs can store any non-reference type, including other structs."

> Ok. It seems like you can't store a reference in a struct.

We're left with just `Multiple` and `Apply`.

And since `Multiple` is only used for return values and expression blocks, we are only left with apply.

> Best practices might be to be able to do a full conversion, should we need it but for now we can just throw an error or panic.

### The mutation of Struct definitions
We should also monitor how the relationships between struct definition and type change or don't change.