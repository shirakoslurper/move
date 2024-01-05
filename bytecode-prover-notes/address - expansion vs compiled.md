Compiled:
`AccountAddress` from `move_core_types`.

Expansion:
```
pub enum Address {

	Numerical(Option<Name>, Spanned<NumericalAddress>),
	
	NamedUnassigned(Name),

}
```