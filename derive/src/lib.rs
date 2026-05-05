// binproto-derive/src/lib.rs
//
// Proc-macro : #[derive(BinProto)]
// Génère automatiquement impl Encode et impl Decode
// en utilisant le crate de Dylann : serialisation_binaire_DD

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Dérive automatique de Encode + Decode pour une struct.
///
/// Exemple :
/// ```ignore
/// #[derive(BinProto)]
/// struct SensorReading {
///     temperature: u32,
///     device_id:   String,
///     is_active:   bool,
/// }
/// ```
#[proc_macro_derive(BinProto)]
pub fn derive_binproto(input: TokenStream) -> TokenStream {
    // 1. Parser le TokenStream en arbre de syntaxe
    let ast = parse_macro_input!(input as DeriveInput);

    // 2. Générer les implémentations
    let expanded = impl_binproto(&ast);

    TokenStream::from(expanded)
}

fn impl_binproto(ast: &DeriveInput) -> TokenStream2 {
    let struct_name = &ast.ident;

    // Récupérer les champs nommés de la struct
    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("BinProto ne supporte que les structs à champs nommés"),
        },
        _ => panic!("BinProto ne supporte que les structs"),
    };

    // ── impl Encode ──────────────────────────────────────────────────────────
    // Pour chaque champ `foo` : self.foo.encode(buf);
    let encode_fields: Vec<TokenStream2> = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                self.#name.encode(buf);
            }
        })
        .collect();

    // ── impl Decode ──────────────────────────────────────────────────────────
    // Pour chaque champ `foo: T` :
    //   let (foo, consumed) = <T as Decode>::decode(&buf[offset..])?;
    //   offset += consumed;
    let decode_fields: Vec<TokenStream2> = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty   = &field.ty;
            quote! {
                let (#name, consumed) = <#ty as binproto::Decode>::decode(&buf[offset..])?;
                offset += consumed;
            }
        })
        .collect();

    // Noms des champs pour construire Self { foo, bar, ... }
    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    // ── Code généré final ────────────────────────────────────────────────────
    quote! {
        impl binproto::Encode for #struct_name {
            fn encode(&self, buf: &mut Vec<u8>) {
                #(#encode_fields)*
            }
        }

        impl binproto::Decode for #struct_name {
            fn decode(buf: &[u8]) -> Result<(Self, usize), binproto_core::DecodeError> {
                let mut offset: usize = 0;

                #(#decode_fields)*

                Ok((Self { #(#field_names),* }, offset))
            }
        }
    }
}
