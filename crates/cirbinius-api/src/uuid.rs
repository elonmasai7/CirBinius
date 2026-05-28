use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn new_v4() -> Self {
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes).expect("rng");
        // Set UUID v4: version bits (4 bits) at positions 12-13 of byte 6
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // Set variant bits (10xx) at positions 8-9 of byte 8
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Self(bytes)
    }

    pub fn parse_str(s: &str) -> Result<Self, String> {
        let s = s.trim().replace('-', "");
        if s.len() != 32 {
            return Err("invalid uuid length".into());
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&s[i*2..i*2+2], 16)
                .map_err(|e| format!("invalid uuid hex: {e}"))?;
        }
        Ok(Self(bytes))
    }

    pub fn to_string(&self) -> String {
        let hex = hex::encode(self.0);
        format!(
            "{}-{}-{}-{}-{}",
            &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32]
        )
    }

    pub fn short(&self) -> String {
        hex::encode(&self.0[..4])
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl serde::Serialize for Uuid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Uuid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

impl std::str::FromStr for Uuid {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_str(s)
    }
}
