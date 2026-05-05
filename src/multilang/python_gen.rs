use crate::schema::{FieldType, Schema};

fn field_type_to_python(t: &FieldType) -> &'static str {
    match t {
        FieldType::U8 | FieldType::U16 | FieldType::U32 | FieldType::U64 => "int",
        FieldType::I32 | FieldType::I64 => "int",
        FieldType::Bool => "bool",
        FieldType::Str => "str",
        FieldType::Bytes => "bytes",
        FieldType::Repeated(_) => "list",
        FieldType::Message(_) => "object",
    }
}

fn generate_encode_field(name: &str, typ: &FieldType) -> String {
    match typ {
        FieldType::Bool => format!(
            "        buf.append(1 if self.{} else 0)\n",
            name
        ),
        FieldType::U8 => format!(
            "        buf.append(self.{} & 0xFF)\n",
            name
        ),
        FieldType::U16 | FieldType::U32 | FieldType::U64 => format!(
            "        val = self.{}\n        while val >= 0x80:\n            buf.append((val & 0x7F) | 0x80)\n            val >>= 7\n        buf.append(val)\n",
            name
        ),
        FieldType::I32 | FieldType::I64 => format!(
            "        val = (self.{0} << 1) ^ (self.{0} >> 31)\n        while val >= 0x80:\n            buf.append((val & 0x7F) | 0x80)\n            val >>= 7\n        buf.append(val)\n",
            name
        ),
        FieldType::Str => format!(
            "        _b = self.{}.encode('utf-8')\n        _l = len(_b)\n        while _l >= 0x80:\n            buf.append((_l & 0x7F) | 0x80)\n            _l >>= 7\n        buf.append(_l)\n        buf.extend(_b)\n",
            name
        ),
        FieldType::Bytes => format!(
            "        _l = len(self.{})\n        while _l >= 0x80:\n            buf.append((_l & 0x7F) | 0x80)\n            _l >>= 7\n        buf.append(_l)\n        buf.extend(self.{})\n",
            name, name
        ),
        FieldType::Repeated(_) => format!(  
            "        _l = len(self.{})\n        buf.append(_l)\n        buf.extend(self.{})\n", name, name
        ),
        FieldType::Message(_) => format!(   
            "        buf.extend(self.{}.encode())\n", name
        ),
    }

}

fn generate_decode_field(name: &str, typ: &FieldType) -> String {
    match typ {
        FieldType::Bool => format!(
            "        {} = buf[offset] != 0\n        offset += 1\n",
            name
        ),
        FieldType::U8 => format!(
            "        {} = buf[offset]\n        offset += 1\n",
            name
        ),
        FieldType::U16 | FieldType::U32 | FieldType::U64 => format!(
            "        {0}, _n = _decode_varint(buf, offset)\n        offset += _n\n",
            name
        ),
        FieldType::I32 | FieldType::I64 => format!(
            "        _raw, _n = _decode_varint(buf, offset)\n        offset += _n\n        {0} = (_raw >> 1) ^ -(_raw & 1)\n",
            name
        ),
        FieldType::Str => format!(
            "        _l, _n = _decode_varint(buf, offset)\n        offset += _n\n        {0} = buf[offset:offset+_l].decode('utf-8')\n        offset += _l\n",
            name
        ),
        FieldType::Bytes => format!(
            "        _l, _n = _decode_varint(buf, offset)\n        offset += _n\n        {0} = bytes(buf[offset:offset+_l])\n        offset += _l\n",
            name
        ),
        FieldType::Repeated(_) => format!(
            "        _l = buf[offset]\n        offset += 1\n        {0} = list(buf[offset:offset+_l])\n        offset += _l\n", name
        ),
        FieldType::Message(_) => format!(  
            "        # nested message decode not yet implemented for {}\n", name
        ),
    }
}

pub fn generate_python(schema: &Schema) -> String {
    let mut out = String::new();

    out.push_str("# Code généré automatiquement par binproto-multilang\n");
    out.push_str("# Ne pas modifier manuellement\n\n");


    out.push_str("def _encode_varint(val, buf):\n");
    out.push_str("    while val >= 0x80:\n");
    out.push_str("        buf.append((val & 0x7F) | 0x80)\n");
    out.push_str("        val >>= 7\n");
    out.push_str("    buf.append(val)\n\n");

    out.push_str("def _decode_varint(buf, offset):\n");
    out.push_str("    result = 0\n");
    out.push_str("    shift = 0\n");
    out.push_str("    while True:\n");
    out.push_str("        b = buf[offset]\n");
    out.push_str("        offset += 1\n");
    out.push_str("        result |= (b & 0x7F) << shift\n");
    out.push_str("        if (b & 0x80) == 0:\n");
    out.push_str("            break\n");
    out.push_str("        shift += 7\n");
    out.push_str("    return result, offset\n\n");

    for msg in schema.messages.values() {
        // Début de la classe
        out.push_str(&format!("class {}:\n", msg.name));

        // __init__
        let params: Vec<String> = msg.fields.iter().map(|f| {
            let default = match &f.typ {
                FieldType::Bool => "False".to_string(),
                FieldType::Str => "''".to_string(),
                FieldType::Bytes => "b''".to_string(),
                FieldType::Repeated(_) => "[]".to_string(),  
                FieldType::Message(_) => "None".to_string(),
                _ => "0".to_string(),
            };
            format!("{}: {} = {}", f.name, field_type_to_python(&f.typ), default)
        }).collect();

        out.push_str(&format!(
            "    def __init__(self, {}):\n",
            params.join(", ")
        ));
        for f in &msg.fields {
            out.push_str(&format!("        self.{0} = {0}\n", f.name));
        }
        out.push('\n');

        // encode
        out.push_str("    def encode(self) -> bytes:\n");
        out.push_str("        buf = bytearray()\n");
        for f in &msg.fields {
            out.push_str(&generate_encode_field(&f.name, &f.typ));
        }
        out.push_str("        return bytes(buf)\n\n");

        // decode
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode(cls, buf: bytes) -> '{}':\n", msg.name));
        out.push_str("        offset = 0\n");
        for f in &msg.fields {
            out.push_str(&generate_decode_field(&f.name, &f.typ));
        }

        let field_names: Vec<String> = msg.fields.iter()
            .map(|f| format!("{}={}", f.name, f.name))
            
            .collect();
        out.push_str(&format!("        return cls({})\n\n", field_names.join(", ")));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldDef, FieldType, MessageDef, Schema};

    #[test]
    fn test_generate_python_sensor() {
        let mut schema = Schema::new();  
        schema.add_message(MessageDef {
            name: "SensorReading".to_string(),
            fields: vec![
                FieldDef { number: 1, typ: FieldType::U32, name: "temperature".to_string(), optional: false },
                FieldDef { number: 2, typ: FieldType::Str, name: "device_id".to_string(), optional: false },
                FieldDef { number: 3, typ: FieldType::Bool, name: "is_active".to_string(), optional: false },
            ],
        });

        let code = generate_python(&schema);
        assert!(code.contains("class SensorReading"));
        assert!(code.contains("def encode"));
        assert!(code.contains("def decode"));
        assert!(code.contains("temperature"));
        assert!(code.contains("device_id"));
    }
}