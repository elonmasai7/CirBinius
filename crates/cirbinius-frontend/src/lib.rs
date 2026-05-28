use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use cirbinius_r1cs::{ParsedR1cs, parse_r1cs_file};
use cirbinius_symbols::{SymbolTable, parse_sym_file};

#[derive(Debug, Clone)]
pub struct R1csBundle {
    pub r1cs: ParsedR1cs,
    pub symbols: SymbolTable,
}

pub fn load_r1cs_bundle(r1cs_path: &Path, sym_path: Option<&Path>) -> Result<R1csBundle> {
    let r1cs = parse_r1cs_file(r1cs_path)?;
    let symbols = if let Some(path) = sym_path {
        parse_sym_file(path)?
    } else {
        SymbolTable {
            by_wire: BTreeMap::new(),
        }
    };

    Ok(R1csBundle { r1cs, symbols })
}
