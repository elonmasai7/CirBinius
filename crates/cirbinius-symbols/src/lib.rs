use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolTable {
    pub by_wire: BTreeMap<u32, String>,
}

impl SymbolTable {
    pub fn get(&self, wire_id: u32) -> Option<&str> {
        self.by_wire.get(&wire_id).map(String::as_str)
    }
}

pub fn parse_sym_file(path: &Path) -> Result<SymbolTable> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read sym file: {}", path.display()))?;
    parse_sym_text(&text)
}

pub fn parse_sym_text(text: &str) -> Result<SymbolTable> {
    let mut by_wire = BTreeMap::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
        if parts.len() < 4 {
            return Err(anyhow!(
                "invalid .sym line {line_number}: expected at least 4 comma-separated columns"
            ));
        }

        let wire_id = parse_wire_id(&parts, line_number)?;
        let name = parts[3].to_string();
        by_wire.insert(wire_id, name);
    }

    Ok(SymbolTable { by_wire })
}

fn parse_wire_id(parts: &[&str], line_number: usize) -> Result<u32> {
    if let Ok(wire_id) = parts[1].parse::<u32>() {
        return Ok(wire_id);
    }

    parts[0]
        .parse::<u32>()
        .map_err(|_| anyhow!("invalid wire id on .sym line {line_number}"))
}

#[cfg(test)]
mod tests {
    use super::parse_sym_text;

    #[test]
    fn parses_sym_text_with_comments() {
        let text = "# comment\n0,1,0,main.a\n1,2,0,main.b\n";
        let table = parse_sym_text(text).expect("sym parse should work");
        assert_eq!(table.get(1), Some("main.a"));
        assert_eq!(table.get(2), Some("main.b"));
    }
}
