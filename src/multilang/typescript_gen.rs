use crate::schema::{FieldType, Schema};

fn field_type_to_typescript(t: &FieldType) -> &'static str {
    match t {

        FieldType::U8 | FieldType::U16 | FieldType::U32 | FieldType::U64 => "number",
        FieldType::I32 | FieldType::I64 => "number",
        FieldType::Bool => "boolean",
        FieldType::Str => "string",
        FieldType::Bytes => "Uint8Array",
        FieldType::Repeated(_) => "number[]",  
        FieldType::Message(_) => "object",
    }
}

fn generate_encode_field_ts(name: &str, typ: &FieldType) -> String {
    match typ {
        FieldType::Bool => format!(

            "    buf.push(msg.{} ? 1 : 0);\n", name
        ),
        FieldType::U8 => format!(
            "    buf.push(msg.{} & 0xFF);\n", name
        ),
        FieldType::U16 | FieldType::U32 | FieldType::U64 => format!(
            "    encodeVarint(msg.{}, buf);\n", name
        ),
        FieldType::I32 | FieldType::I64 => format!(
            "    encodeVarint((msg.{0} << 1) ^ (msg.{0} >> 31), buf);\n", name
        ),
        FieldType::Str => format!(
            "    encodeString(msg.{}, buf);\n", name
        ),
        FieldType::Bytes => format!(
            "    encodeVarint(msg.{}.length, buf);\n    for (const b of msg.{}) buf.push(b);\n",
            name, name
        ),
        FieldType::Repeated(_) => format!(  
            "    buf.push(msg.{}.length);\n    for (const b of msg.{}) buf.push(b);\n",
            name, name
        ),
        FieldType::Message(_) => format!(   
            "    // nested message encode not yet implemented for {}\n", name
        ),
    }
}

fn generate_decode_field_ts(name: &str, typ: &FieldType) -> String {
    match typ {
        FieldType::Bool => format!(

            "    const {} = buf[offset++] !== 0;\n", name
        ),
        FieldType::U8 => format!(
            "    const {} = buf[offset++];\n", name
        ),
        FieldType::U16 | FieldType::U32 | FieldType::U64 => format!(
            "    const [{0}, {0}Len] = readVarint(buf, offset); offset += {0}Len;\n", name
        ),
        FieldType::I32 | FieldType::I64 => format!(
            "    const [raw_{0}, raw_{0}Len] = readVarint(buf, offset); offset += raw_{0}Len;\n    const {0} = (raw_{0} >> 1) ^ -(raw_{0} & 1);\n",
            name
        ),
        FieldType::Str => format!(
            "    const [{0}, {0}Len] = readString(buf, offset); offset += {0}Len;\n", name
        ),
        FieldType::Bytes => format!(

            "    const [{0}Len, {0}LenSize] = readVarint(buf, offset); offset += {0}LenSize;\n    const {0} = buf.slice(offset, offset + {0}Len); offset += {0}Len;\n",
            name
        ),
        FieldType::Repeated(_) => format!(  
            "    const {0}Len = buf[offset++];\n    const {0} = Array.from(buf.slice(offset, offset + {0}Len)); offset += {0}Len;\n",
            name
        ),
        FieldType::Message(_) => format!(   
            "    // nested message decode not yet implemented for {}\n", name
        ),
    }
}

pub fn generate_typescript(schema: &Schema) -> String {
    let mut out = String::new();

    out.push_str("// Code généré automatiquement par binproto-multilang\n");
    out.push_str("// Ne pas modifier manuellement\n\n");

    //. Helpers varint et string
    out.push_str("function encodeVarint(val: number, buf: number[]): void {\n");
    out.push_str("    while (val >= 0x80) {\n");
    out.push_str("        buf.push((val & 0x7F) | 0x80);\n");
    out.push_str("        val >>>= 7;\n");
    out.push_str("    }\n");
    out.push_str("    buf.push(val);\n");     
    out.push_str("}\n\n");

    out.push_str("function readVarint(buf: Uint8Array, offset: number): [number, number] {\n");
    out.push_str("    let result = 0, shift = 0, bytesRead = 0;\n");
    out.push_str("    while (true) {\n");
    out.push_str("        const b = buf[offset + bytesRead++];\n");
    out.push_str("        result |= (b & 0x7F) << shift;\n");
    out.push_str("        if ((b & 0x80) === 0) break;\n");
    out.push_str("        shift += 7;\n");
    out.push_str("    }\n");
    out.push_str("    return [result, bytesRead];\n");
    out.push_str("}\n\n");

    out.push_str("function encodeString(val: string, buf: number[]): void {\n");
    out.push_str("    const encoded = new TextEncoder().encode(val);\n");
    out.push_str("    encodeVarint(encoded.length, buf);\n");
    out.push_str("    for (const b of encoded) buf.push(b);\n");
    out.push_str("}\n\n");

    out.push_str("function readString(buf: Uint8Array, offset: number): [string, number] {\n");
    out.push_str("    const [len, lenSize] = readVarint(buf, offset);\n");
    out.push_str("    const strBytes = buf.slice(offset + lenSize, offset + lenSize + len);\n");
    out.push_str("    return [new TextDecoder().decode(strBytes), lenSize + len];\n");
    out.push_str("}\n\n");

    for (_, msg) in schema.messages.iter() {
        // Interface TypeScript
        out.push_str(&format!("export interface {} {{\n", msg.name));
        for f in &msg.fields { 

            out.push_str(&format!(
                "    {}: {};\n",
                f.name,
                field_type_to_typescript(&f.typ)
            ));
        }
        out.push_str("}\n\n");

        // Fonction encode
        out.push_str(&format!(
            "export function encode{}(msg: {}): Uint8Array {{\n",
            msg.name, msg.name
        ));
        out.push_str("    const buf: number[] = [];\n"); 
        for f in &msg.fields {
            out.push_str(&generate_encode_field_ts(&f.name, &f.typ));
        }
        out.push_str("    return new Uint8Array(buf);\n");
        out.push_str("}\n\n");

        // Fonction decode
        out.push_str(&format!(
            "export function decode{}(buf: Uint8Array): {} {{\n",
            msg.name, msg.name
        ));
        out.push_str("    let offset = 0;\n");
        for f in &msg.fields {
            out.push_str(&generate_decode_field_ts(&f.name, &f.typ));
        }

        let field_names: Vec<String> = msg.fields.iter()
            .map(|f| f.name.clone())
            .collect();
        out.push_str(&format!("    return {{ {} }};\n", field_names.join(", ")));
        out.push_str("}\n\n");
    }

    out
}


#[cfg(test)]
 mod tests {
    use super::*;
    use crate::schema::{FieldDef, FieldType, MessageDef, Schema};

    #[test]
    fn test_generate_typescript_sensor() {
        let mut schema = Schema::new();  // ← corrigé
        schema.add_message(MessageDef {
            name: "SensorReading".to_string(),
            fields: vec![
                FieldDef { number: 1, typ: FieldType::U32, name: "temperature".to_string(), optional: false },
                FieldDef { number: 2, typ: FieldType::Str, name: "device_id".to_string(), optional: false },
                FieldDef { number: 3, typ: FieldType::Bool, name: "is_active".to_string(), optional: false },
            ],
        });

        let code = generate_typescript(&schema);
        assert!(code.contains("export interface SensorReading"));
        assert!(code.contains("export function encodeSensorReading"));
        assert!(code.contains("export function decodeSensorReading"));
        assert!(code.contains("temperature: number"));
        assert!(code.contains("device_id: string"));
    }
    
}