# R1CS Parser v1 Contract

The R1CS parser extracts:

- field modulus
- wire counts
- constraint counts
- public/private/output counts
- sparse A/B/C vectors per constraint

The parser rejects malformed or truncated files and reports offsets for diagnostics.

Semantics target:

`<A, w> * <B, w> = <C, w>` for each constraint.
