The conversion of `parser::SpecBlock` to `expansion::SpecBlock` occurs in `expansion::translate::specs()` and its subroutines.

> I believe that it may be possible to inject a target between or in the parsing and expansion steps. especially since it doesn't seem like the `expansion::SpecBlock`require details about the target (if it is a `Module`).
> 


