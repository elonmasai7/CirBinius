use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

const SECTION_HEADER: u32 = 1;
const SECTION_CONSTRAINTS: u32 = 2;
const SECTION_WIRE_LABELS: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct R1csTerm {
    pub wire_id: u32,
    pub coeff_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct R1csLinearCombination {
    pub terms: Vec<R1csTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct R1csConstraint {
    pub a: R1csLinearCombination,
    pub b: R1csLinearCombination,
    pub c: R1csLinearCombination,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct R1csHeader {
    pub field_size: u32,
    pub field_modulus_hex: String,
    pub wire_count: u32,
    pub public_output_count: u32,
    pub public_input_count: u32,
    pub private_input_count: u32,
    pub label_count: u64,
    pub constraint_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedR1cs {
    pub header: R1csHeader,
    pub constraints: Vec<R1csConstraint>,
    pub wire_labels: Vec<u64>,
}

pub fn parse_r1cs_file(path: &Path) -> Result<ParsedR1cs> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read R1CS file: {}", path.display()))?;
    parse_r1cs_bytes(&bytes)
}

pub fn parse_r1cs_bytes(bytes: &[u8]) -> Result<ParsedR1cs> {
    let mut cursor = Cursor::new(bytes);

    let mut magic = [0_u8; 4];
    cursor
        .read_exact(&mut magic)
        .context("failed to read R1CS magic")?;
    ensure!(&magic == b"r1cs", "invalid R1CS magic");

    let _version = read_u32(&mut cursor, "version")?;
    let section_count = read_u32(&mut cursor, "section count")?;

    let mut header_bytes = None;
    let mut constraints_bytes = None;
    let mut labels_bytes = None;

    for _ in 0..section_count {
        let section_id = read_u32(&mut cursor, "section id")?;
        let section_size = read_u64(&mut cursor, "section size")?;
        let mut section = vec![0_u8; section_size as usize];
        cursor
            .read_exact(&mut section)
            .with_context(|| format!("failed to read section {section_id}"))?;

        match section_id {
            SECTION_HEADER => header_bytes = Some(section),
            SECTION_CONSTRAINTS => constraints_bytes = Some(section),
            SECTION_WIRE_LABELS => labels_bytes = Some(section),
            _ => {}
        }
    }

    let header_blob = header_bytes.context("missing R1CS header section")?;
    let header = parse_header(&header_blob)?;

    let constraints_blob = constraints_bytes.context("missing R1CS constraints section")?;
    let constraints = parse_constraints(
        &constraints_blob,
        header.constraint_count,
        header.field_size,
    )?;

    let wire_labels = if let Some(blob) = labels_bytes {
        parse_wire_labels(&blob, header.wire_count)?
    } else {
        Vec::new()
    };

    Ok(ParsedR1cs {
        header,
        constraints,
        wire_labels,
    })
}

fn parse_header(bytes: &[u8]) -> Result<R1csHeader> {
    let mut cursor = Cursor::new(bytes);
    let field_size = read_u32(&mut cursor, "field size")?;
    let mut field_modulus = vec![0_u8; field_size as usize];
    cursor
        .read_exact(&mut field_modulus)
        .context("failed to read field modulus")?;

    let wire_count = read_u32(&mut cursor, "wire count")?;
    let public_output_count = read_u32(&mut cursor, "public output count")?;
    let public_input_count = read_u32(&mut cursor, "public input count")?;
    let private_input_count = read_u32(&mut cursor, "private input count")?;
    let label_count = read_u64(&mut cursor, "label count")?;
    let constraint_count = read_u32(&mut cursor, "constraint count")?;

    Ok(R1csHeader {
        field_size,
        field_modulus_hex: le_bytes_to_hex(&field_modulus),
        wire_count,
        public_output_count,
        public_input_count,
        private_input_count,
        label_count,
        constraint_count,
    })
}

fn parse_constraints(
    bytes: &[u8],
    constraint_count: u32,
    field_size: u32,
) -> Result<Vec<R1csConstraint>> {
    let mut cursor = Cursor::new(bytes);
    let mut constraints = Vec::with_capacity(constraint_count as usize);
    for _ in 0..constraint_count {
        let a = parse_linear_combination(&mut cursor, field_size)?;
        let b = parse_linear_combination(&mut cursor, field_size)?;
        let c = parse_linear_combination(&mut cursor, field_size)?;
        constraints.push(R1csConstraint { a, b, c });
    }
    Ok(constraints)
}

fn parse_wire_labels(bytes: &[u8], wire_count: u32) -> Result<Vec<u64>> {
    ensure!(
        bytes.len().is_multiple_of(8),
        "invalid wire labels section size: {}",
        bytes.len()
    );
    let mut cursor = Cursor::new(bytes);
    let mut labels = Vec::with_capacity(bytes.len() / 8);
    while (cursor.position() as usize) < bytes.len() {
        labels.push(read_u64(&mut cursor, "wire label")?);
    }

    if !labels.is_empty() {
        ensure!(
            labels.len() == wire_count as usize,
            "wire label count {} does not match wire count {}",
            labels.len(),
            wire_count
        );
    }

    Ok(labels)
}

fn parse_linear_combination(
    cursor: &mut Cursor<&[u8]>,
    field_size: u32,
) -> Result<R1csLinearCombination> {
    let term_count = read_u32(cursor, "linear combination term count")?;
    let mut terms = Vec::with_capacity(term_count as usize);
    for _ in 0..term_count {
        let wire_id = read_u32(cursor, "term wire id")?;
        let mut coeff = vec![0_u8; field_size as usize];
        cursor
            .read_exact(&mut coeff)
            .context("failed to read term coefficient")?;
        terms.push(R1csTerm {
            wire_id,
            coeff_hex: le_bytes_to_hex(&coeff),
        });
    }
    Ok(R1csLinearCombination { terms })
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

#[cfg(test)]
mod tests {
    use super::{
        SECTION_CONSTRAINTS, SECTION_HEADER, SECTION_WIRE_LABELS, le_bytes_to_hex, parse_r1cs_bytes,
    };

    fn bn254_modulus_bytes_le_32() -> [u8; 32] {
        // bn254 modulus as little-endian bytes.
        [
            0x01, 0x00, 0x00, 0xf0, 0x93, 0xf5, 0xe1, 0x43, 0x91, 0x70, 0xb9, 0x79, 0x48, 0xe8,
            0x33, 0x28, 0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1,
            0x72, 0x4e, 0x64, 0x30,
        ]
    }

    fn build_linear_combination(wire_id: u32, coeff: [u8; 32]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&1_u32.to_le_bytes());
        out.extend_from_slice(&wire_id.to_le_bytes());
        out.extend_from_slice(&coeff);
        out
    }

    fn build_sample_r1cs(section_order: &[u32], include_labels: bool) -> Vec<u8> {
        let field_size = 32_u32;
        let wire_count = 4_u32;
        let public_output_count = 0_u32;
        let public_input_count = 1_u32;
        let private_input_count = 2_u32;
        let label_count = 4_u64;
        let constraint_count = 2_u32;

        let mut header = Vec::new();
        header.extend_from_slice(&field_size.to_le_bytes());
        header.extend_from_slice(&bn254_modulus_bytes_le_32());
        header.extend_from_slice(&wire_count.to_le_bytes());
        header.extend_from_slice(&public_output_count.to_le_bytes());
        header.extend_from_slice(&public_input_count.to_le_bytes());
        header.extend_from_slice(&private_input_count.to_le_bytes());
        header.extend_from_slice(&label_count.to_le_bytes());
        header.extend_from_slice(&constraint_count.to_le_bytes());

        let one = {
            let mut coeff = [0_u8; 32];
            coeff[0] = 1;
            coeff
        };
        let zero = [0_u8; 32];

        let mut constraints = Vec::new();
        constraints.extend_from_slice(&build_linear_combination(1, one));
        constraints.extend_from_slice(&build_linear_combination(2, one));
        constraints.extend_from_slice(&build_linear_combination(3, one));

        constraints.extend_from_slice(&build_linear_combination(3, one));
        constraints.extend_from_slice(&build_linear_combination(0, one));
        constraints.extend_from_slice(&build_linear_combination(3, zero));

        let mut labels = Vec::new();
        labels.extend_from_slice(&0_u64.to_le_bytes());
        labels.extend_from_slice(&1_u64.to_le_bytes());
        labels.extend_from_slice(&2_u64.to_le_bytes());
        labels.extend_from_slice(&3_u64.to_le_bytes());

        let mut section_data = Vec::new();
        for section_id in section_order {
            match *section_id {
                SECTION_HEADER => {
                    section_data.extend_from_slice(&SECTION_HEADER.to_le_bytes());
                    section_data.extend_from_slice(&(header.len() as u64).to_le_bytes());
                    section_data.extend_from_slice(&header);
                }
                SECTION_CONSTRAINTS => {
                    section_data.extend_from_slice(&SECTION_CONSTRAINTS.to_le_bytes());
                    section_data.extend_from_slice(&(constraints.len() as u64).to_le_bytes());
                    section_data.extend_from_slice(&constraints);
                }
                SECTION_WIRE_LABELS if include_labels => {
                    section_data.extend_from_slice(&SECTION_WIRE_LABELS.to_le_bytes());
                    section_data.extend_from_slice(&(labels.len() as u64).to_le_bytes());
                    section_data.extend_from_slice(&labels);
                }
                _ => {}
            }
        }

        let mut out = Vec::new();
        out.extend_from_slice(b"r1cs");
        out.extend_from_slice(&1_u32.to_le_bytes());
        out.extend_from_slice(&(section_order.len() as u32).to_le_bytes());
        out.extend_from_slice(&section_data);
        out
    }

    #[test]
    fn converts_little_endian_bytes_to_hex() {
        assert_eq!(le_bytes_to_hex(&[1, 0, 0, 0]), "0x01");
        assert_eq!(le_bytes_to_hex(&[0, 0, 0, 0]), "0x0");
        assert_eq!(le_bytes_to_hex(&[0xff, 0x00]), "0xff");
    }

    #[test]
    fn parses_r1cs_when_sections_are_out_of_order() {
        let bytes = build_sample_r1cs(
            &[SECTION_CONSTRAINTS, SECTION_WIRE_LABELS, SECTION_HEADER],
            true,
        );
        let parsed = parse_r1cs_bytes(&bytes).expect("parser should support section reordering");

        assert_eq!(parsed.header.constraint_count, 2);
        assert_eq!(parsed.constraints.len(), 2);
        assert_eq!(parsed.wire_labels, vec![0, 1, 2, 3]);
        assert_eq!(parsed.constraints[0].a.terms[0].wire_id, 1);
        assert_eq!(parsed.constraints[0].b.terms[0].wire_id, 2);
        assert_eq!(parsed.constraints[0].c.terms[0].wire_id, 3);
    }

    #[test]
    fn parses_r1cs_without_wire_labels_section() {
        let bytes = build_sample_r1cs(&[SECTION_HEADER, SECTION_CONSTRAINTS], false);
        let parsed = parse_r1cs_bytes(&bytes).expect("parser should allow missing labels section");

        assert_eq!(parsed.header.constraint_count, 2);
        assert_eq!(parsed.constraints.len(), 2);
        assert!(parsed.wire_labels.is_empty());
    }
}
