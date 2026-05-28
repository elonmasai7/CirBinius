# CirBinius

CirBinius is an open-source compiler that translates Circom circuits and R1CS artifacts into Binius64-compatible proof circuits.

It allows ZK developers to reuse existing Circom circuits while experimenting with Binius-style binary proof systems. Deterministic artifact contracts, schema-versioned JSON, and SHA-256 content hashing ensure pipeline integrity from source to proof bundle.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI (cirbinius)                          │
│   compile | compile-r1cs | analyze | optimize | lower | prove   │
│   verify | check-witness | doctor | init | inspect | clean     │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                     cirbinius-core (dispatch)                    │
│    Orchestrates all pipeline stages, artifact integrity checks   │
└──────┬──────────┬──────────┬──────────┬──────────┬──────────────┘
       │          │          │          │          │
       ▼          ▼          ▼          ▼          ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│frontend  │ │ normalize│ │optimizer │ │ binius64 │ │ witness  │
│+ r1cs    │ │          │ │+ analyze │ │ lowering │ │ engine   │
│+ symbols │ │          │ │          │ │ backend  │ │          │
└──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
       │          │          │          │          │
       └──────────┴──────────┴──────────┴──────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    cirbinius-artifacts                           │
│  CBIR | ProvePrecheckBundle | ProofBundle | BackendCapabilities │
│  Schema-versioned JSON contracts with sealed SHA-256 hashes     │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  cirbinius-api (HTTP Server)                     │
│  Projects | Uploads | Jobs | Artifacts | Auth | Admin/Stats    │
│  In-memory store | broadcast job queue | rate limiting         │
│  Sandboxed workers with RLIMIT isolation                       │
└─────────────────────────────────────────────────────────────────┘
```

## SDK & Platform Layer

```
┌─────────────────────────────────────────────────────────────────┐
│                        SDK Clients                              │
│  cirbinius-sdk (Rust) | cirbinius-py (Python) | cirbinius-ts   │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                    cirbinius-api (HTTP API)                     │
│  /api/v1/projects | /api/v1/jobs | /api/v1/admin/*            │
└─────────────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                    Compiler Pipeline                            │
│  Frontend → Normalize → CBIR → Optimize → Lower → Prove/Witness│
└─────────────────────────────────────────────────────────────────┘
```

## Pipeline Stages

| Stage | Crate | Description |
|-------|-------|-------------|
| 1. Frontend Loader | `cirbinius-frontend` | Loads `.r1cs` binary + optional `.sym` into a unified bundle |
| 2. R1CS Parser | `cirbinius-r1cs` | Binary `.r1cs` parser (wire count, constraints, A B C matrices) |
| 3. Symbol Resolver | `cirbinius-symbols` | `.sym` text parser mapping signal names to wire indices |
| 4. Constraint Normalizer | `cirbinius-normalize` | Canonicalizes constraint ordering and A-B-C hex values |
| 5. CBIR Builder | `cirbinius-cbir` | Emits CBIR document with deterministic SHA-256 content hash |
| 6. Optimization Passes | `cirbinius-optimizer` | Motif detection (boolean, bit, range, XOR, AND, MUX, Merkle, hash) |
| 7. Analyzer | `cirbinius-optimizer` | Pattern recognition + lowering rules index emission |
| 8. Binius64 Lowering | `cirbinius-binius64` | Classifies constraints into gate families (boolean, range_check, xor, etc.) |
| 9. Witness Engine | `cirbinius-witness` | `.wtns` parser, witness equivalence, constraint replay, snarkjs integration |
| 10. Artifact Contracts | `cirbinius-artifacts` | Schema-versioned JSON types with sealed SHA-256 hashes |
| 11. API Server | `cirbinius-api` | HTTP server with projects, jobs, artifacts, auth, rate limiting |
| 12. Rust SDK | `cirbinius-sdk` | Typed Rust client for the API |
| 13. Python SDK | `cirbinius-py` | Python bindings via C FFI + ctypes |
| 14. TypeScript SDK | `sdk/ts` | TypeScript client for the API |
| 15. Plugin Framework | `cirbinius-plugin` | Plugin system for custom passes |

## CLI Reference

```
cirbinius init         Initialize project scaffold
cirbinius compile      Compile .circom source → CBIR
cirbinius compile-r1cs Compile .r1cs binary → CBIR
cirbinius analyze      Analyze circuit structure + emit lowering rules index
cirbinius optimize     Apply pattern-based optimization passes
cirbinius lower        Lower CBIR → Binius64 artifact
cirbinius prove        Prove precheck (witness gen + bundle emission)
cirbinius verify       Verify proof bundle integrity
cirbinius check-witness Check Circom ↔ Binius witness equivalence
cirbinius doctor       Emit backend capabilities manifest
cirbinius inspect      Inspect circuit metadata
cirbinius benchmark    Benchmark pipeline stages
cirbinius explain      Explain constraint structure
cirbinius clean        Clean build artifacts
```

## API Server

Start the API server:

```bash
# Start with default settings (port 8080)
cargo run --bin cirbinius-api

# Or with custom config
CIRBINIUS_HOST=0.0.0.0 CIRBINIUS_PORT=9090 \
  CIRBINIUS_API_KEY=my-secret-key \
  cargo run --bin cirbinius-api
```

The API serves:
- Developer dashboard at `http://localhost:8080/`
- REST API at `http://localhost:8080/api/v1/*`

See [API Reference](docs/api.md) for full endpoint documentation.

### Docker

```bash
docker build -t cirbinius-api .
docker run -p 8080:8080 cirbinius-api
```

Or with docker-compose:

```bash
docker-compose up
```

## SDK Quick Start

### Rust SDK

```toml
[dependencies]
cirbinius-sdk = { git = "https://github.com/cirbinius/cirbinius" }
```

```rust
use cirbinius_sdk::CirbiniusClient;

let client = CirbiniusClient::new("127.0.0.1", 8080)
    .with_api_key("my-api-key");
let health = client.health().await?;
println!("{health:?}");
```

### Python SDK

```python
from cirbinius import CirbiniusClient

client = CirbiniusClient(host="127.0.0.1", port=8080, api_key="my-api-key")
print(client.health())
```

### TypeScript SDK

```typescript
import { CirbiniusClient } from '@cirbinius/sdk';

const client = new CirbiniusClient('127.0.0.1', 8080, 'my-api-key');
const health = await client.health();
console.log(health);
```

## Quick Start

### 1. Compile from R1CS

```bash
cargo run -- compile-r1cs --r1cs tests/circuits/simple_mul.r1cs \
  --sym tests/circuits/simple_mul.sym --out build/
```

### 2. Compile from Circom Source

```bash
cargo run -- compile tests/circuits/simple_mul.circom --out build/
```

### 3. Analyze and Lower

```bash
cargo run -- analyze --r1cs tests/circuits/simple_mul.r1cs \
  --sym tests/circuits/simple_mul.sym --out build/analysis.json

cargo run -- lower --cbir build/circuit.cbir.json \
  --out build/binius64.json
```

### 4. Prove Precheck

```bash
cargo run -- prove --r1cs tests/circuits/simple_mul.r1cs \
  --sym tests/circuits/simple_mul.sym \
  --wasm build/circom/simple_mul.wasm \
  --input tests/circuits/simple_mul_input.json \
  --out build/ --precheck-only
```

### 5. Verify Bundle

```bash
cargo run -- verify --bundle build/proof_bundle.json
```

### 6. Doctor (Backend Capabilities)

```bash
cargo run -- doctor --out build/backend_capabilities.json
```

## Cross-Platform Installation

### Linux / macOS (Bash)

```bash
curl -sSfL https://github.com/cirbinius/cirbinius/releases/latest/download/install.sh | bash
```

### Windows (PowerShell)

```powershell
iwr -useb https://github.com/cirbinius/cirbinius/releases/latest/download/install.ps1 | iex
```

### Homebrew

```bash
brew install cirbinius/cirbinius/cirbinius
```

### Nix

```bash
nix run github:cirbinius/cirbinius
```

### Cargo

```bash
cargo install --git https://github.com/cirbinius/cirbinius cirbinius-cli cirbinius-api
```

## Workspace Layout

```
cirbinius/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── cirbinius-cli       # CLI binary (clap-based)
│   ├── cirbinius-core      # Command dispatch + pipeline orchestration
│   ├── cirbinius-frontend  # R1CS+SYM loader
│   ├── cirbinius-r1cs      # Binary .r1cs parser
│   ├── cirbinius-symbols   # .sym text parser
│   ├── cirbinius-normalize # Constraint canonicalization
│   ├── cirbinius-cbir      # CBIR IR builder + hash validation
│   ├── cirbinius-optimizer # Motif detection + optimization passes
│   ├── cirbinius-binius64  # Binius64 lowering backend
│   ├── cirbinius-witness   # Witness engine (.wtns, snarkjs, equivalence)
│   ├── cirbinius-artifacts # Artifact contracts (CBIR, bundles, manifests)
│   ├── cirbinius-prover    # Prover runtime scaffold
│   ├── cirbinius-verifier  # Verifier scaffold
│   ├── cirbinius-reports   # Report generation
│   ├── cirbinius-sandbox   # Sandbox/workflow environment
│   ├── cirbinius-api       # API server (hyper + tower)
│   ├── cirbinius-sdk       # Rust SDK client
│   ├── cirbinius-py        # Python bindings (C FFI)
│   ├── cirbinius-plugin    # Plugin framework
│   ├── cirbinius-bench     # Benchmarking harness
│   └── cirbinius-types     # Shared types (Backend, CompileMode, etc.)
├── vendor/httpdate         # Vendored httpdate crate (hyper dep)
├── sdk/
│   └── ts/                 # TypeScript SDK
├── tests/
│   ├── circuits/           # Circuit fixtures (.r1cs, .sym, .wtns, .wasm)
│   ├── golden/             # Golden artifact files
│   ├── fuzz/               # Fuzz test harnesses
│   └── integration/        # Integration test helpers
├── examples/               # Example circuits
├── cirbinius-conformance/  # Conformance test suite
├── docs/
│   ├── contracts/          # Schema-versioned JSON contracts + docs
│   ├── api.md              # API reference
│   ├── sdk.md              # SDK reference
│   ├── architecture.md
│   ├── compiler-pipeline.md
│   ├── cli.md
│   ├── lowering-rules.md
│   ├── cbir-spec.md
│   ├── security.md
│   └── contributing.md
├── web/
│   └── dashboard/          # Developer dashboard (static HTML)
├── scripts/                # Installer scripts + Homebrew formula
└── .github/
    ├── actions/setup-cirbinius/  # GitHub Action
    └── workflows/ci.yml         # CI with schema guard, lint, test, artifact upload
```

## Deterministic Artifact Contracts

Every emitted artifact follows a versioned JSON schema and includes:

| Field | Description |
|-------|-------------|
| `schema_version` | Matches a `docs/contracts/*-vN.schema.json` |
| `toolchain_version` | Crate version that produced the artifact |
| `*_hash` | SHA-256 hash of the hash-stable payload |

Contracts are enforced in CI via `.github/schema_guard.py`.

## License

Apache-2.0
