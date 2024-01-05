

`file_format` notes that `Constant` is a "serialized value along with its type" and that the type/value combo will be deserialized by the loader/evaluator. Looking trhough the loader and interpreter we find that is uses deserialize calls found in `move_vm::types::values` with a `CompiledModule` "object".

[`move_vm::types::values`](https://github.com/move-language/move/blob/c3b79f91f3cd9a1aa16124b0af0af171d970ee71/language/move-vm/types/src/values/values_impl.rs)

We can copy our implementation of deserialization from this or use the components that work or both.

OH SHIT

[BCS](https://docs.rs/bcs/latest/bcs/)
Binary Canonical Serialization was developed for the Diem blockchain for the purposes of:
- providing good performance and concise binary representations.
- supporting a rich set of data types commonly used in Rust.
- enforcing canonical serialization.

The BCS crate provides a Rust implementation of BCS as an encoding format for the Serde library.

According to the `move_vm::types::values` we do not rely on the rust enums representing value to determine how serialization is done.

```
pub fn simple_deserialize(blob: &[u8], layout: &MoveTypeLayout) -> Option<Value> {
	bcs::from_bytes_seed(SeedWrapper { layout }, blob).ok()
}
```

Here, we provide a `MoveTypeLayout` to specify explicit representation of the type layout. Otherwise we would be relying on a consistently implemented internal representation of values (not very robust).

We can find these type layouts specified in `move_core_types::value::MoveTypeLayout`.

Ok it looks like we're gonna have to do a bit of hacky wacky and do this from mostly scratchy batchy.

Most important is implementing `DeserializeSeed` for `SeedWrapper<&MoveTypeLayout>` with `Value = E::Value`.

Hmm, why do we need `SeedWrapper`?

```
SeedWrapper<T> {
	layout: T
}
```
It takes a type parameter - the wrapped value's type.

According to the Rust Design Patterns Book, we can use wrappers to "gracefully handle multiple related types, while minimizing the surface area for memory unsafety".

We can fold all possible interactions into a wrapper type. This means we are only limited to what is provided by the wrapper and nothing else (potentially unsafe actions).

Another reason we might use a wrapper type is to implement the Newtype pattern. But that's with a tuple type... But the principles may be similar. We can implement extra functionality on top of what the base type has on the new/wrapper type.

> I think it may have to do with rust's approach to polymorphism?
> Generics imply static dispatch, `&dyn` would imply dynamic dispatch.

> AHHHH I think i got it. `MoveTypeLayout` is defined in a different module and we cannot implement any additional traits on a struct outside of wher it is defined. The best we can do is create a wrapper type. Implementing deserialize here with a wrapper type makes more sense given that it is required by the immediate context of the Move VM but not necessarily elsewhere (where the implementation of deserialize might differ based on `Value` and the desired return type.)
> 
> As i suspected: "if two crates could implement a trait for the same struct there would be a conflict of which trait implementation to use. With this restriction, Rust guarantees that every (struct, trait) pair will have at most one implementation across all crates."
> 
> Basically it allows our implementation of deserialize on `MoveTypeLayout` to be context/crate dependent.

> "If you find yourself needing to implement an external trait for an external struct you can leverage the [Rust New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)."
>

## Value in value_impl has implemented some stuff

## Generics
The compiler tries to compile the cost as if the generic is all it knows about. It knows the traits that must be implemented by our type (given by the type parameter) but NOTHING about what the type is.
So someway, somehow we have to attempt to coerce our Type parameter into a byte array 


## Something about Rust returning Result
This will exit with a message about exiting with an error code with no useful information whatsoever in the error message or the 

## Deserialization issues
I thought it had something to do with `constant_sig_token()` returning `None` for a couple variants of `file_format::SignatureToken` but that's not the case. I think the issue might be in the implementation of `bcs::from_bytes_seed`. All my testing leads me to believe that everything works properly for `deserialize` in `impl DeserializeSeed` but there's a check of sorts that's dependent on the layout you provide to `from_bytes_seed()` that leads it to return None. 

```
/// Perform a stateful deserialization from a `&[u8]` using the provided `seed`.

pub fn from_bytes_seed<'a, T>(seed: T, bytes: &'a [u8]) -> Result<T::Value>
where
	T: DeserializeSeed<'a>,
{
	let mut deserializer = Deserializer::new(bytes, crate::MAX_CONTAINER_DEPTH);	
	let t = seed.deserialize(&mut deserializer)?;
	deserializer.end().map(move |_| t)
}
```

Comparing this with the call:
```
pub fn simple_deserialize(blob: &[u8], layout: &MoveTypeLayout) -> Option<E::Value> {
	bcs::from_bytes_seed(SeedWrapper { layout }, blob).ok()
}
```

So we called the `deserialize()` we implemented on our seed (type `SeedWrapper`) which we feed the `Deserializer`.

Our call to `simple_deserialize()` with the `Address` variant of `MoveTypeLayout` returns none. Given `.ok()`, this means that our call to `from_bytes_seed()` return an `Err`.

> Ok. I tested it and it is a failure with `from_bytes_seed`.

> We also confirm that `from_bytes_seed` likely successfully calls deserialize. 

So the only fault could be with `deserializer.end().map`.
Our error is a remaining input error.

The following is from [`bcs::de`](https://github.com/zefchain/bcs/blob/main/src/de.rs):
```
/// The `Deserializer::end` method should be called after a type has been
/// fully deserialized. This allows the `Deserializer` to validate that
/// the there are no more bytes remaining in the input stream.

fn end(&mut self) -> Result<()> {
	if self.input.is_empty() {
		Ok(())
	} else {
		Err(Error::RemainingInput)
	}
}
```

> This means we have bytes left that we haven't serialized. How so? What is different?

Okay so after priniting the number of bytes before and the number of bytes after we seem to have gone from 32 to 16 bytes.

AHHH. The issue is because `AccountAddress` has been configured with `LENGTH = 16`. We want it to be 32 and we can set that with the `address32` feature. Features express conditional compilation.

> OK! Setting the feature in Cargo.toml works! 

We've run into another error though.

