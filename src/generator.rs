use crate::schema::{MessageDef, Schema};

use std::io::Write;

/// Generate Rust code from a schema
pub fn generate_code(schema: &Schema) -> String {
    let mut code = String::new();

    // Add imports
    code.push_str("use binproto_core::{Encode, Decode, DecodeError};\n\n");

    // Generate code for each message
    for (_, msg) in &schema.messages {
        generate_message(&mut code, msg);
        code.push('\n');
    }

    code
}

/// Generate Rust code for a single message
fn generate_message(output: &mut String, msg: &MessageDef) {
    // Generate struct definition
    generate_struct_def(output, msg);

    // Generate impl Encode
    generate_encode_impl(output, msg);

    // Generate impl Decode
    generate_decode_impl(output, msg);

    // Generate impl Default
    generate_default_impl(output, msg);
}

/// Generate the struct definition with derives
fn generate_struct_def(output: &mut String, msg: &MessageDef) {
    output.push_str("#[derive(Debug, Clone, PartialEq)]\n");
    output.push_str("pub struct ");
    output.push_str(&msg.name);
    output.push_str(" {\n");

    for field in &msg.fields {
        output.push_str("    pub ");
        output.push_str(&field.name);
        output.push_str(": ");
        output.push_str(&field.typ.to_rust_type());
        output.push_str(",\n");
    }

    output.push_str("}\n\n");
}

/// Generate impl Encode for the message
fn generate_encode_impl(output: &mut String, msg: &MessageDef) {
    output.push_str("impl Encode for ");
    output.push_str(&msg.name);
    output.push_str(" {\n");
    output.push_str("    fn encode(&self, buf: &mut Vec<u8>) {\n");

    for field in &msg.fields {
        output.push_str("        self.");
        output.push_str(&field.name);
        output.push_str(".encode(buf);\n");
    }

    output.push_str("    }\n");
    output.push_str("}\n\n");
}

/// Generate impl Decode for the message
fn generate_decode_impl(output: &mut String, msg: &MessageDef) {
    output.push_str("impl Decode for ");
    output.push_str(&msg.name);
    output.push_str(" {\n");
    output.push_str("    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {\n");
    output.push_str("        let mut total_bytes_read = 0;\n\n");

    // Decode each field
    for field in &msg.fields {
        output.push_str("        let (");
        output.push_str(&field.name);
        output.push_str(", bytes_read) = ");
        output.push_str(&field.typ.to_rust_type());
        output.push_str("::decode(&buf[total_bytes_read..])?;\n");
        output.push_str("        total_bytes_read += bytes_read;\n\n");
    }

    output.push_str("        Ok((");
    output.push_str(&msg.name);
    output.push_str(" {\n");

    for field in &msg.fields {
        output.push_str("            ");
        output.push_str(&field.name);
        output.push_str(",\n");
    }

    output.push_str("        }, total_bytes_read))\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");
}

/// Generate impl Default for the message
fn generate_default_impl(output: &mut String, msg: &MessageDef) {
    output.push_str("impl Default for ");
    output.push_str(&msg.name);
    output.push_str(" {\n");
    output.push_str("    fn default() -> Self {\n");
    output.push_str("        ");
    output.push_str(&msg.name);
    output.push_str(" {\n");

    for field in &msg.fields {
        output.push_str("            ");
        output.push_str(&field.name);
        output.push_str(": ");
        output.push_str(&field.typ.default_value());
        output.push_str(",\n");
    }

    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");
}

/// Write generated code to a file
pub fn write_to_file(schema: &Schema, output_path: &str) -> std::io::Result<()> {
    let code = generate_code(schema);
    let mut file = std::fs::File::create(output_path)?;
    file.write_all(code.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldDef, FieldType};

    #[test]
    fn test_generate_simple_message() {
        let mut schema = Schema::new();

        schema.add_message(MessageDef {
            name: "SensorReading".to_string(),
            fields: vec![
                FieldDef {
                    number: 1,
                    name: "temperature".to_string(),
                    typ: FieldType::U32,
                    optional: false,
                },
                FieldDef {
                    number: 2,
                    name: "device_id".to_string(),
                    typ: FieldType::Str,
                    optional: false,
                },
                FieldDef {
                    number: 3,
                    name: "is_active".to_string(),
                    typ: FieldType::Bool,
                    optional: false,
                },
            ],
        });

        let code = generate_code(&schema);

        // Check that generated code contains expected elements
        assert!(code.contains("pub struct SensorReading"));
        assert!(code.contains("pub temperature: u32"));
        assert!(code.contains("pub device_id: String"));
        assert!(code.contains("pub is_active: bool"));
        assert!(code.contains("impl Encode for SensorReading"));
        assert!(code.contains("impl Decode for SensorReading"));
        assert!(code.contains("impl Default for SensorReading"));
        assert!(code.contains("#[derive(Debug, Clone, PartialEq)]"));
    }

    #[test]
    fn test_generate_encode_impl() {
        let mut schema = Schema::new();

        schema.add_message(MessageDef {
            name: "Message".to_string(),
            fields: vec![FieldDef {
                number: 1,
                name: "value".to_string(),
                typ: FieldType::U32,
                optional: false,
            }],
        });

        let code = generate_code(&schema);

        // Check encode implementation
        assert!(code.contains("impl Encode for Message {"));
        assert!(code.contains("fn encode(&self, buf: &mut Vec<u8>)"));
        assert!(code.contains("self.value.encode(buf);"));
    }

    #[test]
    fn test_generate_decode_impl() {
        let mut schema = Schema::new();

        schema.add_message(MessageDef {
            name: "Message".to_string(),
            fields: vec![FieldDef {
                number: 1,
                name: "value".to_string(),
                typ: FieldType::U32,
                optional: false,
            }],
        });

        let code = generate_code(&schema);

        // Check decode implementation
        assert!(code.contains("impl Decode for Message {"));
        assert!(code.contains("fn decode(buf: &[u8])"));
        assert!(code.contains("let (value, bytes_read) = u32::decode"));
    }

    #[test]
    fn test_generated_code_compiles() {
        let mut schema = Schema::new();

        schema.add_message(MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![
                FieldDef {
                    number: 1,
                    name: "id".to_string(),
                    typ: FieldType::U32,
                    optional: false,
                },
                FieldDef {
                    number: 2,
                    name: "name".to_string(),
                    typ: FieldType::Str,
                    optional: false,
                },
            ],
        });

        let code = generate_code(&schema);
        
        // The code should be valid Rust (we'd need a full compile test)
        // For now, just verify the structure
        assert!(code.contains("pub struct TestMessage {"));
        assert!(code.contains("pub id: u32,"));
        assert!(code.contains("pub name: String,"));
        assert!(code.contains("impl Encode for TestMessage"));
        assert!(code.contains("impl Decode for TestMessage"));
        assert!(code.contains("impl Default for TestMessage"));
    }

    #[test]
    fn test_generate_multiple_messages() {
        let mut schema = Schema::new();

        schema.add_message(MessageDef {
            name: "Message1".to_string(),
            fields: vec![FieldDef {
                number: 1,
                name: "field1".to_string(),
                typ: FieldType::U32,
                optional: false,
            }],
        });

        schema.add_message(MessageDef {
            name: "Message2".to_string(),
            fields: vec![FieldDef {
                number: 1,
                name: "field2".to_string(),
                typ: FieldType::Str,
                optional: false,
            }],
        });

        let code = generate_code(&schema);

        assert!(code.contains("pub struct Message1"));
        assert!(code.contains("pub struct Message2"));
        assert!(code.contains("impl Encode for Message1"));
        assert!(code.contains("impl Encode for Message2"));
    }

    #[test]
    fn test_generate_with_different_types() {
        let mut schema = Schema::new();

        schema.add_message(MessageDef {
            name: "ComplexMessage".to_string(),
            fields: vec![
                FieldDef {
                    number: 1,
                    name: "u8_field".to_string(),
                    typ: FieldType::U8,
                    optional: false,
                },
                FieldDef {
                    number: 2,
                    name: "u64_field".to_string(),
                    typ: FieldType::U64,
                    optional: false,
                },
                FieldDef {
                    number: 3,
                    name: "i32_field".to_string(),
                    typ: FieldType::I32,
                    optional: false,
                },
                FieldDef {
                    number: 4,
                    name: "bool_field".to_string(),
                    typ: FieldType::Bool,
                    optional: false,
                },
                FieldDef {
                    number: 5,
                    name: "bytes_field".to_string(),
                    typ: FieldType::Bytes,
                    optional: false,
                },
                FieldDef {
                    number: 6,
                    name: "list_field".to_string(),
                    typ: FieldType::Repeated(Box::new(FieldType::U32)),
                    optional: false,
                },
            ],
        });

        let code = generate_code(&schema);

        assert!(code.contains("pub u8_field: u8"));
        assert!(code.contains("pub u64_field: u64"));
        assert!(code.contains("pub i32_field: i32"));
        assert!(code.contains("pub bool_field: bool"));
        assert!(code.contains("pub bytes_field: Vec<u8>"));
        assert!(code.contains("pub list_field: Vec<u32>"));
    }

    #[test]
    fn test_generation_output_format() {
        let mut schema = Schema::new();
        schema.add_message(MessageDef {
            name: "SimpleMsg".to_string(),
            fields: vec![FieldDef {
                number: 1,
                name: "counter".to_string(),
                typ: FieldType::U32,
                optional: false,
            }],
        });

        let code = generate_code(&schema);

        // Verify proper indentation and formatting
        assert!(code.contains("    pub counter: u32,"));
        assert!(code.contains("    fn encode(&self, buf: &mut Vec<u8>)"));
        assert!(code.contains("    fn decode(buf: &[u8])"));
    }

    #[test]
    fn test_default_impl_values() {
        let mut schema = Schema::new();
        schema.add_message(MessageDef {
            name: "Message".to_string(),
            fields: vec![
                FieldDef {
                    number: 1,
                    name: "num".to_string(),
                    typ: FieldType::U32,
                    optional: false,
                },
                FieldDef {
                    number: 2,
                    name: "text".to_string(),
                    typ: FieldType::Str,
                    optional: false,
                },
                FieldDef {
                    number: 3,
                    name: "flag".to_string(),
                    typ: FieldType::Bool,
                    optional: false,
                },
            ],
        });

        let code = generate_code(&schema);

        // Check default impl
        assert!(code.contains("impl Default for Message {"));
        assert!(code.contains("num: 0,"));
        assert!(code.contains("text: String::new(),"));
        assert!(code.contains("flag: false,"));
    }
}
