use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail, ensure};
use cirbinius_r1cs::{ParsedR1cs, R1csConstraint, R1csLinearCombination};
use cirbinius_symbols::SymbolTable;
use num_bigint::BigUint;
use num_traits::{Num, Zero};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const WTNS_SECTION_HEADER: u32 = 1;
const WTNS_SECTION_WITNESS: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedWitness {
    pub field_size: u32,
    pub field_modulus_hex: String,
    pub witness_len: u32,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WitnessValueMismatch {
    pub wire_id: u32,
    pub signal: Option<String>,
    pub circom_value_hex: String,
    pub binius_value_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConstraintReplayFailure {
    pub constraint_id: u64,
    pub signal_path: Option<String>,
    pub circom_residual_hex: String,
    pub binius_residual_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WitnessCheckReport {
    pub equivalent: bool,
    pub compared_wire_count: u32,
    pub value_mismatch_count: usize,
    pub constraint_failure_count: usize,
    pub value_mismatches: Vec<WitnessValueMismatch>,
    pub constraint_failures: Vec<ConstraintReplayFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessGenerationRequest {
    pub snarkjs_bin: String,
    pub wasm_path: PathBuf,
    pub input_json_path: PathBuf,
    pub output_wtns_path: PathBuf,
}

pub fn parse_wtns_file(path: &Path) -> Result<ParsedWitness> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read wtns file: {}", path.display()))?;
    parse_wtns_bytes(&bytes)
}

pub fn generate_wtns_with_snarkjs(request: &WitnessGenerationRequest) -> Result<()> {
    let output = Command::new(&request.snarkjs_bin)
        .arg("wtns")
        .arg("calculate")
        .arg(&request.wasm_path)
        .arg(&request.input_json_path)
        .arg(&request.output_wtns_path)
        .output()
        .with_context(|| format!("failed to execute snarkjs binary '{}'", request.snarkjs_bin))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "snarkjs witness generation failed (status: {}):\nstdout:\n{}\nstderr:\n{}",
            output.status.code().map_or_else(
                || "terminated by signal".to_string(),
                |code| code.to_string()
            ),
            stdout,
            stderr
        );
    }

    if !request.output_wtns_path.exists() {
        bail!(
            "snarkjs did not produce expected witness file at {}",
            request.output_wtns_path.display()
        );
    }

    Ok(())
}

pub fn parse_wtns_bytes(bytes: &[u8]) -> Result<ParsedWitness> {
    let mut cursor = Cursor::new(bytes);

    let mut magic = [0_u8; 4];
    cursor
        .read_exact(&mut magic)
        .context("failed to read wtns magic")?;
    ensure!(&magic == b"wtns", "invalid wtns magic");

    let _version = read_u32(&mut cursor, "wtns version")?;
    let section_count = read_u32(&mut cursor, "wtns section count")?;

    let mut header_bytes = None;
    let mut witness_bytes = None;

    for _ in 0..section_count {
        let section_id = read_u32(&mut cursor, "wtns section id")?;
        let section_size = read_u64(&mut cursor, "wtns section size")?;
        let mut section = vec![0_u8; section_size as usize];
        cursor
            .read_exact(&mut section)
            .with_context(|| format!("failed to read wtns section {section_id}"))?;
        match section_id {
            WTNS_SECTION_HEADER => header_bytes = Some(section),
            WTNS_SECTION_WITNESS => witness_bytes = Some(section),
            _ => {}
        }
    }

    let header_bytes = header_bytes.context("missing wtns header section")?;
    let (field_size, field_modulus_hex, witness_len) = parse_wtns_header(&header_bytes)?;
    let witness_bytes = witness_bytes.context("missing wtns witness section")?;
    let values = parse_wtns_values(&witness_bytes, field_size, witness_len)?;

    Ok(ParsedWitness {
        field_size,
        field_modulus_hex,
        witness_len,
        values,
    })
}

pub fn parse_binius_witness_json_file(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read binius witness json file: {}",
            path.display()
        )
    })?;
    parse_binius_witness_json_text(&text)
}

pub fn parse_binius_witness_json_text(text: &str) -> Result<Vec<String>> {
    let value: Value = serde_json::from_str(text).context("invalid binius witness json")?;
    parse_binius_witness_value(value)
}

pub fn check_witness_equivalence(
    parsed_r1cs: &ParsedR1cs,
    symbols: &SymbolTable,
    circom_witness: &[String],
    binius_witness: &[String],
) -> Result<WitnessCheckReport> {
    let prime = parse_biguint(&parsed_r1cs.header.field_modulus_hex)?;
    let compared_wire_count = parsed_r1cs.header.wire_count;
    let expected_len = compared_wire_count as usize;

    ensure!(
        circom_witness.len() >= expected_len,
        "circom witness length {} is smaller than expected wire count {}",
        circom_witness.len(),
        expected_len
    );
    ensure!(
        binius_witness.len() >= expected_len,
        "binius witness length {} is smaller than expected wire count {}",
        binius_witness.len(),
        expected_len
    );

    let mut value_mismatches = Vec::new();
    for wire_id in 0..compared_wire_count {
        let idx = wire_id as usize;
        let circom = parse_biguint(&circom_witness[idx])? % &prime;
        let binius = parse_biguint(&binius_witness[idx])? % &prime;
        if circom != binius {
            value_mismatches.push(WitnessValueMismatch {
                wire_id,
                signal: symbols.get(wire_id).map(str::to_string),
                circom_value_hex: biguint_to_hex(&circom),
                binius_value_hex: biguint_to_hex(&binius),
            });
        }
    }

    let mut constraint_failures = Vec::new();
    for (index, constraint) in parsed_r1cs.constraints.iter().enumerate() {
        let circom_residual = replay_constraint(constraint, circom_witness, &prime)?;
        let binius_residual = replay_constraint(constraint, binius_witness, &prime)?;
        let circom_ok = circom_residual.is_zero();
        let binius_ok = binius_residual.is_zero();

        if !circom_ok || !binius_ok || circom_residual != binius_residual {
            constraint_failures.push(ConstraintReplayFailure {
                constraint_id: index as u64 + 1,
                signal_path: constraint_signal_hint(constraint, symbols),
                circom_residual_hex: biguint_to_hex(&circom_residual),
                binius_residual_hex: biguint_to_hex(&binius_residual),
            });
        }
    }

    value_mismatches.sort_by_key(|mismatch| mismatch.wire_id);
    constraint_failures.sort_by_key(|failure| failure.constraint_id);

    let equivalent = value_mismatches.is_empty() && constraint_failures.is_empty();
    Ok(WitnessCheckReport {
        equivalent,
        compared_wire_count,
        value_mismatch_count: value_mismatches.len(),
        constraint_failure_count: constraint_failures.len(),
        value_mismatches,
        constraint_failures,
    })
}

fn parse_wtns_header(bytes: &[u8]) -> Result<(u32, String, u32)> {
    let mut cursor = Cursor::new(bytes);
    let field_size = read_u32(&mut cursor, "wtns field size")?;
    let mut field_modulus = vec![0_u8; field_size as usize];
    cursor
        .read_exact(&mut field_modulus)
        .context("failed to read wtns field modulus")?;
    let witness_len = read_u32(&mut cursor, "wtns witness length")?;
    Ok((field_size, le_bytes_to_hex(&field_modulus), witness_len))
}

fn parse_wtns_values(bytes: &[u8], field_size: u32, witness_len: u32) -> Result<Vec<String>> {
    let expected_size = witness_len as usize * field_size as usize;
    ensure!(
        bytes.len() >= expected_size,
        "wtns witness section size {} is smaller than expected {}",
        bytes.len(),
        expected_size
    );

    let mut cursor = Cursor::new(bytes);
    let mut values = Vec::with_capacity(witness_len as usize);
    for _ in 0..witness_len {
        let mut scalar = vec![0_u8; field_size as usize];
        cursor
            .read_exact(&mut scalar)
            .context("failed to read witness scalar")?;
        values.push(canonical_hex(&le_bytes_to_hex(&scalar)));
    }
    Ok(values)
}

fn parse_binius_witness_value(value: Value) -> Result<Vec<String>> {
    match value {
        Value::Array(items) => items.iter().map(json_scalar_to_hex).collect(),
        Value::Object(object) => {
            if let Some(witness) = object.get("witness") {
                if let Value::Array(items) = witness {
                    return items.iter().map(json_scalar_to_hex).collect();
                }
                return Err(anyhow!("'witness' key must contain an array"));
            }

            let mut by_index = BTreeMap::new();
            for (key, val) in object {
                let index = key
                    .parse::<usize>()
                    .with_context(|| format!("invalid witness index key '{key}'"))?;
                by_index.insert(index, json_scalar_to_hex(&val)?);
            }
            if by_index.is_empty() {
                return Ok(Vec::new());
            }

            let max = *by_index
                .keys()
                .last()
                .ok_or_else(|| anyhow!("failed to determine witness max index"))?;
            let mut out = vec!["0x0".to_string(); max + 1];
            for (idx, val) in by_index {
                out[idx] = val;
            }
            Ok(out)
        }
        _ => Err(anyhow!(
            "binius witness json must be an array or object with 'witness' field"
        )),
    }
}

fn json_scalar_to_hex(value: &Value) -> Result<String> {
    match value {
        Value::String(str_value) => {
            let parsed = parse_biguint(str_value)?;
            Ok(biguint_to_hex(&parsed))
        }
        Value::Number(number) => {
            let as_u64 = number
                .as_u64()
                .ok_or_else(|| anyhow!("json number is not an unsigned integer"))?;
            Ok(biguint_to_hex(&BigUint::from(as_u64)))
        }
        _ => Err(anyhow!("unsupported witness scalar json type")),
    }
}

fn replay_constraint(
    constraint: &R1csConstraint,
    witness: &[String],
    prime: &BigUint,
) -> Result<BigUint> {
    let a = eval_linear_combination(&constraint.a, witness, prime)?;
    let b = eval_linear_combination(&constraint.b, witness, prime)?;
    let c = eval_linear_combination(&constraint.c, witness, prime)?;

    let product = (a * b) % prime;
    let residual = if product >= c {
        (product - c) % prime
    } else {
        (product + prime - c) % prime
    };
    Ok(residual)
}

fn eval_linear_combination(
    linear: &R1csLinearCombination,
    witness: &[String],
    prime: &BigUint,
) -> Result<BigUint> {
    let mut acc = BigUint::zero();
    for term in &linear.terms {
        let idx = term.wire_id as usize;
        if idx >= witness.len() {
            bail!(
                "witness does not contain wire {} (len {})",
                term.wire_id,
                witness.len()
            );
        }

        let coeff = parse_biguint(&term.coeff_hex)? % prime;
        let value = parse_biguint(&witness[idx])? % prime;
        let term_value = (coeff * value) % prime;
        acc = (acc + term_value) % prime;
    }
    Ok(acc)
}

fn constraint_signal_hint(constraint: &R1csConstraint, symbols: &SymbolTable) -> Option<String> {
    first_symbol_in_linear(&constraint.c, symbols)
        .or_else(|| first_symbol_in_linear(&constraint.a, symbols))
        .or_else(|| first_symbol_in_linear(&constraint.b, symbols))
}

fn first_symbol_in_linear(linear: &R1csLinearCombination, symbols: &SymbolTable) -> Option<String> {
    linear
        .terms
        .iter()
        .find_map(|term| symbols.get(term.wire_id).map(str::to_string))
}

fn parse_biguint(value: &str) -> Result<BigUint> {
    let raw = value.trim();
    if let Some(stripped) = raw.strip_prefix("0x") {
        return BigUint::from_str_radix(stripped, 16)
            .map_err(|_| anyhow!("failed to parse hex scalar '{}'", value));
    }
    BigUint::from_str_radix(raw, 10).map_err(|_| anyhow!("failed to parse scalar '{}'", value))
}

fn read_u32(cursor: &mut Cursor<&[u8]>, name: &str) -> Result<u32> {
    let mut buf = [0_u8; 4];
    cursor
        .read_exact(&mut buf)
        .with_context(|| format!("failed to read {name}"))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(cursor: &mut Cursor<&[u8]>, name: &str) -> Result<u64> {
    let mut buf = [0_u8; 8];
    cursor
        .read_exact(&mut buf)
        .with_context(|| format!("failed to read {name}"))?;
    Ok(u64::from_le_bytes(buf))
}

fn le_bytes_to_hex(le_bytes: &[u8]) -> String {
    let mut be = le_bytes.to_vec();
    be.reverse();
    let first_non_zero = be.iter().position(|byte| *byte != 0);
    let bytes = if let Some(idx) = first_non_zero {
        &be[idx..]
    } else {
        return "0x0".to_string();
    };

    let mut out = String::from("0x");
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn biguint_to_hex(value: &BigUint) -> String {
    if value.is_zero() {
        "0x0".to_string()
    } else {
        format!("0x{value:x}")
    }
}

fn canonical_hex(value: &str) -> String {
    let without_prefix = value.trim_start_matches("0x");
    let trimmed = without_prefix.trim_start_matches('0');
    if trimmed.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_binius_witness_json_text, parse_wtns_bytes};

    #[test]
    fn parses_binius_witness_from_array_json() {
        let values = parse_binius_witness_json_text("[\"0x1\", 2, \"3\"]")
            .expect("json witness parse should work");
        assert_eq!(values, vec!["0x1", "0x2", "0x3"]);
    }

    #[test]
    fn parses_minimal_wtns_bytes() {
        let field_size = 32_u32;
        let witness_len = 2_u32;

        let mut header = Vec::new();
        header.extend_from_slice(&field_size.to_le_bytes());
        let mut modulus = [0_u8; 32];
        modulus[0] = 7;
        header.extend_from_slice(&modulus);
        header.extend_from_slice(&witness_len.to_le_bytes());

        let mut witness = Vec::new();
        let mut one = [0_u8; 32];
        one[0] = 1;
        let mut three = [0_u8; 32];
        three[0] = 3;
        witness.extend_from_slice(&one);
        witness.extend_from_slice(&three);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"wtns");
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u32.to_le_bytes());

        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&header);

        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&(witness.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&witness);

        let parsed = parse_wtns_bytes(&bytes).expect("wtns parse should work");
        assert_eq!(parsed.witness_len, 2);
        assert_eq!(parsed.values, vec!["0x1", "0x3"]);
    }
}
