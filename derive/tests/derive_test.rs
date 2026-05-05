// binproto-derive/tests/derive_test.rs
//
// Tests d'intégration pour #[derive(BinProto)]
// Utilise les vrais traits de Dylann (serialisation_binaire_DD)

use binproto::{Decode, Encode};
use binproto_derive::BinProto;

// ── Struct principale du sujet ────────────────────────────────────────────────

#[derive(BinProto, Debug, PartialEq)]
struct SensorReading {
    temperature: u32,
    device_id:   String,
    is_active:   bool,
}

#[test]
fn test_sensor_reading_roundtrip() {
    let original = SensorReading {
        temperature: 2350,
        device_id:   String::from("capteur-42"),
        is_active:   true,
    };

    let mut buf = Vec::new();
    original.encode(&mut buf);

    let (decoded, consumed) = SensorReading::decode(&buf).expect("decode failed");

    assert_eq!(decoded, original);
    assert_eq!(consumed, buf.len());
}

// ── u32 + varint de Dylann ────────────────────────────────────────────────────

#[derive(BinProto, Debug, PartialEq)]
struct Counter {
    value: u32,
}

#[test]
fn test_varint_u32_roundtrip() {
    // Dylann encode u32 en varint : 300 → [0xAC, 0x02]
    let c = Counter { value: 300 };
    let mut buf = Vec::new();
    c.encode(&mut buf);
    assert_eq!(buf, vec![0xAC, 0x02]); // vérifie le format varint exact
    let (decoded, _) = Counter::decode(&buf).unwrap();
    assert_eq!(decoded, c);
}

#[test]
fn test_max_u32_roundtrip() {
    let c = Counter { value: u32::MAX };
    let mut buf = Vec::new();
    c.encode(&mut buf);
    let (decoded, _) = Counter::decode(&buf).unwrap();
    assert_eq!(decoded.value, u32::MAX);
}

// ── i32 avec zigzag (spécifique à Dylann) ────────────────────────────────────

#[derive(BinProto, Debug, PartialEq)]
struct Signed {
    altitude: i32,
}

#[test]
fn test_i32_negatif_roundtrip() {
    let s = Signed { altitude: -100 };
    let mut buf = Vec::new();
    s.encode(&mut buf);
    let (decoded, _) = Signed::decode(&buf).unwrap();
    assert_eq!(decoded, s);
}

// ── Plusieurs types ensemble ──────────────────────────────────────────────────

#[derive(BinProto, Debug, PartialEq)]
struct Packet {
    id:      u32,
    label:   String,
    enabled: bool,
    count:   u64,
}

#[test]
fn test_packet_roundtrip() {
    let pkt = Packet {
        id:      1,
        label:   String::from("hello"),
        enabled: false,
        count:   1_000_000,
    };

    let mut buf = Vec::new();
    pkt.encode(&mut buf);

    let (decoded, consumed) = Packet::decode(&buf).unwrap();
    assert_eq!(decoded, pkt);
    assert_eq!(consumed, buf.len());
}

// ── String vide ───────────────────────────────────────────────────────────────

#[test]
fn test_string_vide() {
    let original = SensorReading {
        temperature: 0,
        device_id:   String::new(),
        is_active:   false,
    };
    let mut buf = Vec::new();
    original.encode(&mut buf);
    let (decoded, _) = SensorReading::decode(&buf).unwrap();
    assert_eq!(decoded, original);
}

// ── Buffer trop court → erreur ────────────────────────────────────────────────

#[test]
fn test_buffer_trop_court() {
    let original = SensorReading {
        temperature: 42,
        device_id:   String::from("abc"),
        is_active:   true,
    };
    let mut buf = Vec::new();
    original.encode(&mut buf);

    let short = &buf[..2]; // tronqué
    let result = SensorReading::decode(short);
    assert!(result.is_err());
}

// ── Vec<T> ────────────────────────────────────────────────────────────────────

#[derive(BinProto, Debug, PartialEq)]
struct WithVec {
    values: Vec<u8>,
}

#[test]
fn test_vec_roundtrip() {
    let w = WithVec { values: vec![10, 20, 30] };
    let mut buf = Vec::new();
    w.encode(&mut buf);
    let (decoded, _) = WithVec::decode(&buf).unwrap();
    assert_eq!(decoded, w);
}
