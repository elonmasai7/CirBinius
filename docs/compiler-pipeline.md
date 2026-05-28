# Compiler Pipeline

CirBinius compiler stages:

1. Frontend Loader
2. R1CS Parser
3. Symbol Resolver
4. Constraint Normalizer
5. CBIR Generation
6. Optimization Passes
7. Binius64 Lowering
8. Proof Runtime Generation
9. Verifier Generation
10. Reporting and Benchmarking

Each stage accepts a validated input contract and emits deterministic outputs.
