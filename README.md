# BinProto

> Framework de protocole binaire réseau en Rust  

---

## 1. Présentation

BinProto est un framework minimaliste de définition et d'implémentation de protocoles binaires réseau. Il permet de :

- Décrire des messages dans un fichier schéma `.bps`
- Générer automatiquement le code Rust correspondant
- Communiquer entre machines via TCP de façon ultra-performante

Par rapport au format JSON classique :

| Métrique | BinProto | JSON |
|---|---|---|
| Taille d'un message type | 26 octets | 67 octets |
| Vitesse d'encodage | ~87 ns | ~318 ns |
| Ratio | **2.6x plus compact** | **3.6x plus lent** |

---

## 2. Architecture du projet

```
BinProto/
├── src/
│   ├── lib.rs                      # Sérialisation binaire (varint, zigzag, Encode/Decode)
│   ├── schema.rs                   # Parser du format .bps
│   ├── generator.rs                # Générateur de code Rust
│   ├── server.rs                   # Serveur TCP asynchrone (tokio)
│   ├── client.rs                   # Client TCP asynchrone
│   ├── debugger.rs                 # Interface TUI (ratatui)
│   ├── codegen.rs                  # Logique du binaire codegen
│   ├── bin/
│   │   ├── server.rs               # main() serveur
│   │   ├── client.rs               # main() client
│   │   └── debugger.rs             # main() débogueur
│   │   
│   └── multilang/
│       ├── mod.rs
│       ├── python_gen.rs           # Génération Python
│       ├── typescript_gen.rs       # Génération TypeScript
│       └── examples.rs             # Exemples multilangage
├── derive/
│   ├── Cargo.toml                  # proc-macro = true (obligatoire)
│   └── src/
│       └── lib.rs                  # #[derive(BinProto)]
├── benches/
│   └── encoding_benchmark.rs       # Benchmarks BinProto vs JSON
├── schema/
│   └── messages.bps                # Schéma exemple
├── generated/
│   └── messages.rs                 # Code généré automatiquement
└── Cargo.toml
```

---

## 3. Modules en détail

### 3.1 Sérialisation binaire — `src/lib.rs`

Cœur du projet. Encode/décode tous les types primitifs Rust en binaire compact.

| Type | Encodage |
|---|---|
| `u8` | 1 octet direct |
| `u32`, `u64` | Varint LEB128 (1 octet si valeur < 128) |
| `i32`, `i64` | Zigzag + varint |
| `bool` | 1 octet (0 ou 1) |
| `String` | Longueur varint + octets UTF-8 |
| `Vec<T>` | Longueur varint + éléments encodés |

Traits définis :
```rust
pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>);
}

pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError>;
}
```

---

### 3.2 Parser de schéma — `src/schema.rs`

Lit un fichier `.bps` et produit en mémoire un `Schema` contenant tous les messages.

Format du fichier `.bps` :
```
message SensorReading {
  1: u32 temperature;
  2: string device_id;
  3: optional bool is_active;
  4: Vec<u32> ids;
}
```

Types supportés : `u8`, `u16`, `u32`, `u64`, `i32`, `i64`, `bool`, `string`, `bytes`, `Vec<T>`, `optional T`, références à d'autres messages.

---

### 3.3 Générateur de code — `src/generator.rs`

Génère automatiquement du code Rust valide depuis un schéma. Pour chaque message, il produit :

- `pub struct MessageName { ... }`
- `impl Encode for MessageName { ... }`
- `impl Decode for MessageName { ... }`
- `impl Default for MessageName { ... }`

```bash
cargo run --bin codegen -- schema/messages.bps --output generated/messages.rs
```

---

### 3.4 Serveur TCP — `src/server.rs`

Serveur asynchrone tokio, multi-clients. Format de trame :

```
[ 4 octets longueur (big-endian) ][ 2 octets type_id ][ N octets payload ]
```

Chaque type de message est routé vers son propre handler enregistré.

```bash
cargo run --bin server   # port 8989 par défaut
```

---

### 3.5 Client TCP — `src/client.rs`

Client asynchrone avec deux modes :

- `send_raw()` — envoie et attend la réponse
- `send_one_way()` — envoie sans attendre (fire-and-forget)

```bash
cargo run --bin client
```

---

### 3.6 Débogueur TUI — `src/debugger.rs`

Interface terminal graphique (ratatui + crossterm) affichant les messages en temps réel.

| Touche | Action |
|---|---|
| `↑` / `↓` | Naviguer dans la liste |
| `q` | Quitter |

```bash
cargo run --bin debugger
```

---

### 3.7 Génération multilangage — `src/multilang/`

Génère du code équivalent dans d'autres langages depuis le même schéma :

- `python_gen.rs` — Classes Python avec `encode()` et `decode()`
- `typescript_gen.rs` — Interfaces TypeScript avec `encodeFoo()` / `decodeFoo()`

---

### 3.8 Proc-macro — `derive/`

Fournit `#[derive(BinProto)]` qui génère automatiquement `Encode` + `Decode` pour n'importe quelle struct :

```rust
#[derive(BinProto, Debug, PartialEq)]
struct SensorReading {
    temperature: u32,
    device_id:   String,
    is_active:   bool,
}
```

> Les proc-macros Rust doivent obligatoirement être dans une crate séparée avec `proc-macro = true`. C'est pourquoi `derive/` reste un sous-dossier indépendant avec son propre `Cargo.toml`.

---

### 3.9 Benchmarks — `benches/`

Benchmarks criterion sur 4 groupes :

- `encode` — encodage seul
- `decode` — décodage seul
- `roundtrip` — encode + decode
- `size_bytes` — comparaison de taille

```bash
cargo bench
# Rapports HTML : target/criterion/report/index.html
```

---

## 4. Compilation et utilisation

### Prérequis

- Rust ≥ 1.70 (édition 2021)
- `cargo` installé

### Commandes

```bash
# Compiler
cargo build

# Tests
cargo test

# Benchmarks
cargo bench

# Et pour voir les résultats des benchmarks une fois terminés :
open target/criterion/report/index.html

# Démarrer le serveur (port 8989)
cargo run --bin server

# Démarrer le client
cargo run --bin client

# Ouvrir le débogueur TUI
cargo run --bin debugger

# Générer la documentation HTML
cargo doc --open

```

---

## 5. Warnings connus et corrections

### Warning 1 — `unused doc comment` dans `python_gen.rs`

```
warning: unused doc comment
  --> src/multilang/python_gen.rs:137:9
  |
  |         /// encode
  |         ^^^^^^^^^^
  = help: use `//` for a plain comment
```

**Cause** : un `///` utilisé dans une expression, là où rustdoc ne génère pas de doc.  
**Correction** : ligne 137 de `src/multilang/python_gen.rs`, remplacer :

```rust
/// encode
```

par :

```rust
// encode
```

---

### Warning 2 — `unused import: HandlerFn` dans `src/bin/server.rs`

```
warning: unused import: `HandlerFn`
  --> src/bin/server.rs:3:44
  |
  |     use binproto::server::{BinProtoServer, HandlerFn};
  |                                            ^^^^^^^^^
```

**Cause** : `HandlerFn` importé mais non utilisé dans le `main()` simplifié.  
**Correction** : retirer `HandlerFn` de l'import :

```rust
// AVANT
use binproto::server::{BinProtoServer, HandlerFn};

// APRÈS
use binproto::server::BinProtoServer;
```

---

## 6. Consolidation effectuée

Le projet original était un workspace avec 8 crates séparées. Voici ce qui a été fusionné :

| Ancienne crate | Nouveau fichier | Modifications |
|---|---|---|
| `serialisation_binaire_DD` | `src/lib.rs` | Aucune |
| `parser_schema` | `src/schema.rs` | Aucune |
| `generateur_de_code` | `src/generator.rs` | `binproto_schema::` → `crate::schema::` |
| `binproto-runtime` (server) | `src/server.rs` | Aucune |
| `binproto-runtime` (client) | `src/client.rs` | Import commenté corrigé |
| `binproto-monitor` (4 fichiers) | `src/debugger.rs` | Fusion app + events + ui + main |
| `generateur_de_code/bin.rs` | `src/bin/codegen.rs` | `binproto_schema/codegen::` → `binproto::` |
| `binproto-multilang` | `src/multilang/` | `List` → `Repeated`, imports, HashMap |
| `benchmarks` | `benches/` | `binproto_core::` → `binproto::` |
| `binproto-derive` | `derive/` (séparée) | `binproto_core::` → `binproto::` |

---

## 7. Dépendances

| Dépendance | Version | Usage |
|---|---|---|
| `tokio` | 1.* | Runtime asynchrone |
| `ratatui` | 0.26.3 | Interface TUI du débogueur |
| `crossterm` | 0.27.0 | Gestion terminal pour ratatui |
| `criterion` | 0.5 *(dev)* | Benchmarks avec rapports HTML |
| `serde` + `serde_json` | 1.* *(dev)* | Comparaison JSON dans les benchmarks |
| `proc-macro2`, `quote`, `syn` | 1.*, 2.* | Crate derive — génération de code macro |

---
