# CirBinius Conformance Suite

This directory contains test circuits, inputs, and expected outputs for validating CirBinius compiler correctness.

## Structure

```
circuits/          - Circom circuits (.circom)
inputs/            - Input JSON files
expected/          - Expected output hashes
reports/           - Generated conformance reports
```

## Running

```bash
cirbinius-api      # Start the API server
# or use the CLI directly:
cirbinius compile circuits/multiplier.circom --backend binius64 --out build/
cirbinius prove build/ --input inputs/multiplier.json --proof proof.bin --public public.json
cirbinius verify build/ --proof proof.bin --public public.json
```

## Test Circuits

- `multiplier.circom` - Basic multiplication circuit
- `range_check.circom` - Range check gadget
- More circuits coming soon
