use binproto::generator::write_to_file;  // ← crate:: devient binproto::
use binproto::schema::Schema; 
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <schema.bps> [--output OUTPUT_FILE]", args[0]);
        eprintln!("Example: {} schema.bps --output src/generated.rs", args[0]);
        std::process::exit(1);
    }

    let schema_path = &args[1];
    
    // chemin de sortie par défaut
    let mut output_path = "generated.rs".to_string();

    // Parse optionnel --resultat pour spécifier le chemin de sortie du code généré
    if args.len() >= 4 && args[2] == "--output" {
        output_path = args[3].clone();
    }

    // lire fichier de schema
    let schema_text = match fs::read_to_string(schema_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading schema file '{}': {}", schema_path, e);
            std::process::exit(1);
        }
    };

    // Parse schema
    let schema = match Schema::parse(&schema_text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing schema: {}", e);
            std::process::exit(1);
        }
    };

    // Ecrire le code généré 
    match write_to_file(&schema, &output_path) {
        Ok(_) => {
            println!("Successfully generated code to: {}", output_path);
            println!("Generated {} messages", schema.messages.len());
        }
        Err(e) => {
            eprintln!("Error writing output file '{}': {}", output_path, e);
            std::process::exit(1);
        }
    }
}
