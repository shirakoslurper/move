Translation happens in 3 steps:
1. Declaration Analysis: we collect all information about structs, functions, spec functions, etc. in a module. We don;t yet analyze the bodies, conditions, and invariants.
2. Definition Analysis: we visit the definitions we have skipped in the previous step.
3. Population: we popoulate the global environment with the information fomr this module.