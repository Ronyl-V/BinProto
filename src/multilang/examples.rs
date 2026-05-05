// src/multilang/examples.rs
use crate::schema::{FieldDef, FieldType, MessageDef, Schema};  
use crate::multilang::python_gen::generate_python;              
use crate::multilang::typescript_gen::generate_typescript;      

pub fn run_example() {  
    let mut schema = Schema::new();  
    schema.add_message(MessageDef {
        name: "SensorReading".to_string(),
        fields: vec![
            FieldDef {
                number: 1,
                typ: FieldType::U32,
                name: "temperature".to_string(),
                optional: false,  
            },
            FieldDef {
                number: 2,
                typ: FieldType::Str,
                name: "device_id".to_string(),
                optional: false, 
            },
            FieldDef {
                number: 3,
                typ: FieldType::Bool,
                name: "is_active".to_string(),
                optional: false,  
            },
        ],
    });

    // Génère le code Python
    println!("=== CODE PYTHON GÉNÉRÉ ===\n");
    let python_code = generate_python(&schema);
    println!("{}", python_code);

    // Génère le code TypeScript
    println!("=== CODE TYPESCRIPT GÉNÉRÉ ===\n");
    let ts_code = generate_typescript(&schema);
    println!("{}", ts_code);
}