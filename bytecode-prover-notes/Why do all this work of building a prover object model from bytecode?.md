This will or should allow us to build a full prover object model without specifying additional dependencies and including them.

Now, source code does specify dependencies as downloading source also downloads the Move.toml. This means that we *should* be able to compile with the specified dependencies.

However, this is not the case. Depending on how dependencies we specified in `Move.toml`, we might get a changed dependecy or even one thta introduced a breaking change.

> I tried compiling the downloaded source code for PancakeSwap exchange as it failed to compiled due to the introduction of `inline` in its function definitions.

Builde a prover object model from bytecode (if it works) allows us to successsfully build a model without relying on un reliable things like dependencies. It also saves us some precious compilation time. Additionally, it works with the compiled code as is. If later introductions are mode to the compiler, such that the bytecode differs, starting from source would be largely unreliable. (Especially since the prover object model is bytecode dependent).