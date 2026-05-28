# CLI Reference

## Usage

```
cirbinius <COMMAND> [OPTIONS]
```

## Global Options

| Option | Description |
|--------|-------------|
| `--project-root <PATH>` | Project root directory (default: `.`) |

## Commands

### `init`
Initialize a CirBinius project scaffold in the current directory.

### `compile`
Compile a `.circom` source file into CBIR.

| Option | Required | Description |
|--------|----------|-------------|
| `<SOURCE>` | yes | Path to `.circom` source file |
| `--main <NAME>` | no | Main component name |
| `--include <PATH>` | no | Circom include path (repeatable) |
| `--out <DIR>` | yes | Output directory |
| `--circom-bin <BIN>` | no | Circom binary path (default: `circom`) |

### `compile-r1cs`
Compile a `.r1cs` binary (with optional `.sym`) into CBIR.

| Option | Required | Description |
|--------|----------|-------------|
| `--r1cs <PATH>` | yes | Path to `.r1cs` file |
| `--sym <PATH>` | no | Path to `.sym` file |
| `--out <DIR>` | yes | Output directory |

### `analyze`
Analyze circuit structure and emit a lowering rules index.

| Option | Required | Description |
|--------|----------|-------------|
| `--r1cs <PATH>` | yes | Path to `.r1cs` file |
| `--sym <PATH>` | no | Path to `.sym` file |
| `--out <PATH>` | yes | Output path for analysis report |
| `--optimized-binary` | no | Enable optimized binary mode |

### `optimize`
Apply pattern-based optimization passes.

| Option | Required | Description |
|--------|----------|-------------|
| `--r1cs <PATH>` | yes | Path to `.r1cs` file |
| `--sym <PATH>` | no | Path to `.sym` file |
| `--out <DIR>` | yes | Output directory |
| `--optimized-binary` | no | Enable optimized binary mode |

### `lower`
Lower a CBIR document into a Binius64 artifact.

| Option | Required | Description |
|--------|----------|-------------|
| `--cbir <PATH>` | yes | Path to CBIR JSON file |
| `--out <PATH>` | yes | Output path for lowering artifact |

### `prove`
Generate witness and emit a proof bundle.

| Option | Required | Description |
|--------|----------|-------------|
| `--r1cs <PATH>` | yes | Path to `.r1cs` file |
| `--sym <PATH>` | no | Path to `.sym` file |
| `--wasm <PATH>` | yes | Path to `.wasm` file |
| `--input <PATH>` | yes | Path to input JSON |
| `--out <DIR>` | yes | Output directory |
| `--snarkjs-bin <BIN>` | no | SnarkJS binary path (default: `snarkjs`) |
| `--binius-witness <PATH>` | no | Optional Binius witness JSON for equivalence check |
| `--precheck-report <PATH>` | no | Precheck report output path |
| `--precheck-only` | no | Run in precheck-only mode (skip real proof generation) |
| `--backend-capabilities <PATH>` | no | Path to backend capabilities manifest |

### `verify`
Verify proof bundle integrity.

| Option | Required | Description |
|--------|----------|-------------|
| `--bundle <PATH>` | yes | Path to proof bundle JSON |

### `check-witness`
Check witness equivalence between Circom and Binius.

| Option | Required | Description |
|--------|----------|-------------|
| `--r1cs <PATH>` | yes | Path to `.r1cs` file |
| `--sym <PATH>` | no | Path to `.sym` file |
| `--circom-witness <PATH>` | yes | Path to Circom `.wtns` file |
| `--binius-witness <PATH>` | yes | Path to Binius witness JSON |
| `--out <PATH>` | yes | Output path for witness report |

### `doctor`
Emit a backend capabilities manifest.

| Option | Required | Description |
|--------|----------|-------------|
| `--out <PATH>` | no | Output path (default: `build/backend_capabilities.json`) |

### `inspect`, `benchmark`, `explain`, `clean`
Scaffold commands with implementation in progress.
