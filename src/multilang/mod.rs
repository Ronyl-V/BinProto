// src/multilang/mod.rs
pub mod python_gen;
pub mod typescript_gen;
pub mod examples;
// Types représentant le schéma (copiés de Fadimatou)
#[derive(Debug, Clone)]
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
    List(Box<FieldType>),
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub number: u32,
    pub typ: FieldType,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MessageDef {
    pub name: String, 
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
pub struct Schema {
    
    pub messages: Vec<MessageDef>,
}