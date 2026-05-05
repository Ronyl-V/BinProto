use std::collections::HashMap;

/// Represents the different field types in a BinProto schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
    Bool,
    Str,
    Bytes,
    /// A field referring to another message type
    Message(String),
    /// A repeated field (list)
    Repeated(Box<FieldType>),
}

impl FieldType {
    /// Convert to corresponding Rust type string
    pub fn to_rust_type(&self) -> String {
        match self {
            FieldType::U8 => "u8".to_string(),
            FieldType::U16 => "u16".to_string(),
            FieldType::U32 => "u32".to_string(),
            FieldType::U64 => "u64".to_string(),
            FieldType::I32 => "i32".to_string(),
            FieldType::I64 => "i64".to_string(),
            FieldType::Bool => "bool".to_string(),
            FieldType::Str => "String".to_string(),
            FieldType::Bytes => "Vec<u8>".to_string(),
            FieldType::Message(name) => name.clone(),
            FieldType::Repeated(inner) => format!("Vec<{}>", inner.to_rust_type()),
        }
    }

    /// Get default value for this type
    pub fn default_value(&self) -> String {
        match self {
            FieldType::U8 | FieldType::U16 | FieldType::U32 | FieldType::U64 
            | FieldType::I32 | FieldType::I64 => "0".to_string(),
            FieldType::Bool => "false".to_string(),
            FieldType::Str => "String::new()".to_string(),
            FieldType::Bytes => "Vec::new()".to_string(),
            FieldType::Message(_) => "Default::default()".to_string(),
            FieldType::Repeated(_) => "Vec::new()".to_string(),
        }
    }
}

/// Represents a single field in a message definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDef {
    /// Field number (for binary encoding)
    pub number: u32,
    /// Field name
    pub name: String,
    /// Field type
    pub typ: FieldType,
    /// Whether field is optional
    pub optional: bool,
}

/// Represents a message definition in the schema
#[derive(Debug, Clone)]
pub struct MessageDef {
    /// Message name
    pub name: String,
    /// Fields in this message
    pub fields: Vec<FieldDef>,
}

/// Complete schema containing all message definitions
#[derive(Debug, Clone)]
pub struct Schema {
    /// Map of message name -> MessageDef
    pub messages: HashMap<String, MessageDef>,
}

impl Schema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Schema {
            messages: HashMap::new(),
        }
    }

    /// Add a message definition to the schema
    pub fn add_message(&mut self, msg: MessageDef) {
        self.messages.insert(msg.name.clone(), msg);
    }

    /// Get a message definition by name
    pub fn get_message(&self, name: &str) -> Option<&MessageDef> {
        self.messages.get(name)
    }

    /// Parse a schema from text format
    /// Simple format:
    /// message SensorReading {
    ///   1: u32 temperature;
    ///   2: string device_id;
    ///   3: bool is_active;
    /// }
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut schema = Schema::new();
        let mut lines = input.lines().peekable();

        while lines.peek().is_some() {
            let line = lines.next().unwrap().trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse message declaration
            if line.starts_with("message ") {
                let rest = &line[8..];
                let name = rest
                    .split_whitespace()
                    .next()
                    .ok_or("Expected message name")?
                    .to_string();

                let mut fields = Vec::new();

                // Parse fields until we hit closing brace
                while let Some(next_line) = lines.peek() {
                    let trimmed = next_line.trim();
                    if trimmed == "}" {
                        lines.next();
                        break;
                    }

                    if trimmed.is_empty() || trimmed.starts_with("//") {
                        lines.next();
                        continue;
                    }

                    let field_line = lines.next().unwrap().trim().to_string();
                    if let Ok(field) = Self::parse_field(&field_line) {
                        fields.push(field);
                    }
                }

                schema.add_message(MessageDef { name, fields });
            }
        }

        Ok(schema)
    }

    /// Parse a single field definition
    fn parse_field(line: &str) -> Result<FieldDef, String> {
        // Remove trailing semicolon
        let line = line.trim_end_matches(';').trim();

        // Format: "1: u32 temperature" or "1: optional u32 temperature" or "1: Vec<u32> ids"
        // Strategy: split by colon first, then parse the type and name
        
        let colon_parts: Vec<&str> = line.splitn(2, ':').collect();
        if colon_parts.len() != 2 {
            return Err("Missing colon in field definition".to_string());
        }

        let number = colon_parts[0]
            .trim()
            .parse::<u32>()
            .map_err(|_| "Invalid field number")?;

        let rest = colon_parts[1].trim();
        
        // Now parse "optional? type name"
        // We need to find where the type ends and the name begins
        // Types can be: u32, Vec<u32>, CustomType, etc.
        
        let optional = rest.starts_with("optional ");
        let type_and_name = if optional {
            &rest[9..] // Skip "optional "
        } else {
            rest
        };

        // Find the field name and type
        // If type contains <, we need to find the matching >
        let (typ_str, name) = if type_and_name.contains('<') {
            // Find the end of Vec<...>
            if let Some(close_bracket) = type_and_name.find('>') {
                let type_part = &type_and_name[..=close_bracket];
                let remaining = type_and_name[close_bracket + 1..].trim();
                (type_part.to_string(), remaining.to_string())
            } else {
                return Err("Unmatched < in type definition".to_string());
            }
        } else {
            // Simple case: just split by whitespace
            let parts: Vec<&str> = type_and_name.split_whitespace().collect();
            if parts.len() < 2 {
                return Err("Expected type and field name".to_string());
            }
            (parts[0].to_string(), parts[1].to_string())
        };

        let typ = if typ_str.starts_with("Vec<") && typ_str.ends_with(">") {
            let inner_type_str = &typ_str[4..typ_str.len()-1];
            let inner = match inner_type_str.trim() {
                "u8" => FieldType::U8,
                "u16" => FieldType::U16,
                "u32" => FieldType::U32,
                "u64" => FieldType::U64,
                "i32" => FieldType::I32,
                "i64" => FieldType::I64,
                "bool" => FieldType::Bool,
                "string" => FieldType::Str,
                s => FieldType::Message(s.to_string()),
            };
            FieldType::Repeated(Box::new(inner))
        } else {
            match typ_str.as_str() {
                "u8" => FieldType::U8,
                "u16" => FieldType::U16,
                "u32" => FieldType::U32,
                "u64" => FieldType::U64,
                "i32" => FieldType::I32,
                "i64" => FieldType::I64,
                "bool" => FieldType::Bool,
                "string" => FieldType::Str,
                "bytes" => FieldType::Bytes,
                s => FieldType::Message(s.to_string()),
            }
        };

        Ok(FieldDef {
            number,
            name,
            typ,
            optional,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_to_rust() {
        assert_eq!(FieldType::U32.to_rust_type(), "u32");
        assert_eq!(FieldType::Str.to_rust_type(), "String");
        assert_eq!(FieldType::Bool.to_rust_type(), "bool");
        assert_eq!(FieldType::Bytes.to_rust_type(), "Vec<u8>");
        assert_eq!(
            FieldType::Repeated(Box::new(FieldType::U32)).to_rust_type(),
            "Vec<u32>"
        );
    }

    #[test]
    fn test_field_type_default() {
        assert_eq!(FieldType::U32.default_value(), "0");
        assert_eq!(FieldType::Bool.default_value(), "false");
        assert_eq!(FieldType::Str.default_value(), "String::new()");
    }

    #[test]
    fn test_schema_parse_simple() {
        let schema_text = r#"
message SensorReading {
  1: u32 temperature;
  2: string device_id;
  3: bool is_active;
}
        "#;

        let schema = Schema::parse(schema_text).expect("Failed to parse schema");
        assert_eq!(schema.messages.len(), 1);

        let msg = schema.get_message("SensorReading").expect("Message not found");
        assert_eq!(msg.name, "SensorReading");
        assert_eq!(msg.fields.len(), 3);
        assert_eq!(msg.fields[0].name, "temperature");
        assert_eq!(msg.fields[0].typ, FieldType::U32);
        assert_eq!(msg.fields[1].name, "device_id");
        assert_eq!(msg.fields[1].typ, FieldType::Str);
    }

    #[test]
    fn test_schema_parse_with_comments() {
        let schema_text = r#"
// This is a sensor reading message
message SensorReading {
  // Temperature in Celsius
  1: u32 temperature;
  2: string device_id;
}
        "#;

        let schema = Schema::parse(schema_text).expect("Failed to parse schema");
        assert_eq!(schema.messages.len(), 1);
    }

    #[test]
    fn test_field_def_creation() {
        let field = FieldDef {
            number: 1,
            name: "temperature".to_string(),
            typ: FieldType::U32,
            optional: false,
        };
        assert_eq!(field.number, 1);
        assert_eq!(field.name, "temperature");
    }

    #[test]
    fn test_schema_multiple_messages() {
        let schema_text = r#"
message SensorReading {
  1: u32 temperature;
  2: string device_id;
}

message Config {
  1: string name;
  2: u32 version;
}
        "#;

        let schema = Schema::parse(schema_text).expect("Failed to parse schema");
        assert_eq!(schema.messages.len(), 2);
        assert!(schema.get_message("SensorReading").is_some());
        assert!(schema.get_message("Config").is_some());
    }

    #[test]
    fn test_schema_with_vec_type() {
        let schema_text = r#"
message WithList {
  1: string name;
  2: Vec<u32> ids;
}
        "#;

        let schema = Schema::parse(schema_text).expect("Failed to parse schema");
        assert_eq!(schema.messages.len(), 1);
        
        let msg = schema.get_message("WithList").expect("Message not found");
        assert_eq!(msg.fields.len(), 2);
        assert_eq!(msg.fields[1].name, "ids");
        assert_eq!(msg.fields[1].typ, FieldType::Repeated(Box::new(FieldType::U32)));
    }
}
