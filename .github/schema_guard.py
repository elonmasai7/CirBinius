#!/usr/bin/env python3

import json
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
ARTIFACTS_RS = ROOT / "crates" / "cirbinius-artifacts" / "src" / "lib.rs"
CONTRACTS_DIR = ROOT / "docs" / "contracts"


def fail(message: str) -> None:
    print(f"[schema-guard] ERROR: {message}")
    sys.exit(1)


def read_constants() -> dict[str, str]:
    text = ARTIFACTS_RS.read_text(encoding="utf-8")
    pattern = re.compile(r'pub const ([A-Z0-9_]+): &str = "([^"]+)";')
    constants = {}
    for name, value in pattern.findall(text):
        if name.endswith("_SCHEMA_VERSION"):
            constants[name] = value
    return constants


def slug_and_version(schema_version: str) -> tuple[str, str]:
    match = re.fullmatch(r"([a-z0-9-]+)/v([0-9]+)", schema_version)
    if not match:
        fail(
            f"schema version '{schema_version}' must match '<slug>/v<major>' (e.g. cbir/v1)"
        )
    slug, major = match.group(1), match.group(2)
    return slug, major


def assert_doc_and_schema_files(constants: dict[str, str]) -> None:
    if not constants:
        fail("no *_SCHEMA_VERSION constants found")

    for name, schema_version in constants.items():
        slug, major = slug_and_version(schema_version)
        base = f"{slug}-v{major}"
        md_path = CONTRACTS_DIR / f"{base}.md"
        schema_path = CONTRACTS_DIR / f"{base}.schema.json"

        if not md_path.exists():
            fail(
                f"missing contract doc for {name}={schema_version}: expected {md_path.relative_to(ROOT)}"
            )
        if not schema_path.exists():
            fail(
                f"missing JSON schema for {name}={schema_version}: expected {schema_path.relative_to(ROOT)}"
            )

        md_text = md_path.read_text(encoding="utf-8")
        if schema_version not in md_text:
            fail(
                f"contract doc {md_path.relative_to(ROOT)} must mention schema version '{schema_version}'"
            )

        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        props = schema.get("properties", {})
        schema_const = None

        # Direct top-level schema_version field.
        if isinstance(props.get("schema_version"), dict):
            schema_const = props.get("schema_version", {}).get("const")

        # Nested metadata.schema_version field (used by CBIR).
        if schema_const is None and isinstance(props.get("metadata"), dict):
            metadata_props = props.get("metadata", {}).get("properties", {})
            if isinstance(metadata_props, dict) and isinstance(
                metadata_props.get("schema_version"), dict
            ):
                schema_const = metadata_props.get("schema_version", {}).get("const")

        if schema_const != schema_version:
            fail(
                f"{schema_path.relative_to(ROOT)} schema_version.const is '{schema_const}', expected '{schema_version}'"
            )


def main() -> None:
    constants = read_constants()
    assert_doc_and_schema_files(constants)
    print("[schema-guard] OK: schema constants, docs, and JSON schemas are in sync")


if __name__ == "__main__":
    main()
