# CBIR Specification

CBIR is the Circom-Binius Intermediate Representation.

Core node classes:

- constants
- witness variables
- public inputs
- private inputs
- outputs
- linear combinations
- multiplication constraints
- boolean/range/equality constraints

CBIR is serialized as JSON for interoperability and will have a binary encoding for high-throughput pipelines.
