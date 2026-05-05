// benches/encoding_benchmark.rs

use binproto::{Decode, Encode};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorReading {
    pub temperature: u32,
    pub device_id: String,
    pub is_active: bool,
    pub timestamp: u64,
}

impl Encode for SensorReading {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.temperature.encode(buf);
        self.device_id.encode(buf);
        self.is_active.encode(buf);
        self.timestamp.encode(buf);
    }
}

impl Decode for SensorReading {
    fn decode(buf: &[u8]) -> Result<(Self, usize), binproto::DecodeError> {
        let mut offset = 0;

        let (temperature, n) = u32::decode(&buf[offset..])?;
        offset += n;

        let (device_id, n) = String::decode(&buf[offset..])?;
        offset += n;

        let (is_active, n) = bool::decode(&buf[offset..])?;
        offset += n;

        let (timestamp, n) = u64::decode(&buf[offset..])?;
        offset += n;

        Ok((SensorReading { temperature, device_id, is_active, timestamp }, offset))
    }
}

// DatabaseResponse — struct supplémentaire pour diversifier les benchmarks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseResponse {
    pub status_code: u32,
    pub message: String,
    pub success: bool,
}

impl Encode for DatabaseResponse {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.status_code.encode(buf);
        self.message.encode(buf);
        self.success.encode(buf);
    }
}

impl Decode for DatabaseResponse {
    fn decode(buf: &[u8]) -> Result<(Self, usize), binproto::DecodeError> {
        let mut offset = 0;

        let (status_code, n) = u32::decode(&buf[offset..])?;
        offset += n;

        let (message, n) = String::decode(&buf[offset..])?;
        offset += n;

        let (success, n) = bool::decode(&buf[offset..])?;
        offset += n;

        Ok((DatabaseResponse { status_code, message, success }, offset))
    }
}

// =====================================================================
// Données de test réalistes
// =====================================================================

fn sample_sensor() -> SensorReading {
    SensorReading {
        temperature: 25,
        device_id: "capteur-A-zone-3".to_string(),
        is_active: true,
        timestamp: 1714000000,
    }
}

fn sample_db_response() -> DatabaseResponse {
    DatabaseResponse {
        status_code: 200,
        message: "Requête traitée avec succès".to_string(),
        success: true,
    }
}

// =====================================================================
// GROUPE 1 : Encodage — BinProto vs JSON
// =====================================================================

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    let sensor = sample_sensor();
    let mut bp_buf = Vec::new();
    sensor.encode(&mut bp_buf);
    let json_buf = serde_json::to_vec(&sensor).unwrap();

    group.throughput(Throughput::Bytes(bp_buf.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("binproto", "SensorReading"),
        &sensor,
        |b, s| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(32);
                s.encode(&mut buf);
                buf
            })
        },
    );

    group.throughput(Throughput::Bytes(json_buf.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("json", "SensorReading"),
        &sensor,
        |b, s| b.iter(|| serde_json::to_vec(s).unwrap()),
    );

    let db = sample_db_response();
    let mut bp_buf2 = Vec::new();
    db.encode(&mut bp_buf2);
    let json_buf2 = serde_json::to_vec(&db).unwrap();

    group.throughput(Throughput::Bytes(bp_buf2.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("binproto", "DatabaseResponse"),
        &db,
        |b, d| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(32);
                d.encode(&mut buf);
                buf
            })
        },
    );

    group.throughput(Throughput::Bytes(json_buf2.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("json", "DatabaseResponse"),
        &db,
        |b, d| b.iter(|| serde_json::to_vec(d).unwrap()),
    );

    group.finish();
}

// =====================================================================
// GROUPE 2 : Décodage — BinProto vs JSON
// =====================================================================

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    let sensor = sample_sensor();
    let mut bp_bytes = Vec::new();
    sensor.encode(&mut bp_bytes);
    let json_bytes = serde_json::to_vec(&sensor).unwrap();

    group.throughput(Throughput::Bytes(bp_bytes.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("binproto", "SensorReading"),
        &bp_bytes,
        |b, bytes| b.iter(|| SensorReading::decode(bytes).unwrap()),
    );

    group.throughput(Throughput::Bytes(json_bytes.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("json", "SensorReading"),
        &json_bytes,
        |b, bytes| b.iter(|| serde_json::from_slice::<SensorReading>(bytes).unwrap()),
    );

    let db = sample_db_response();
    let mut bp_bytes2 = Vec::new();
    db.encode(&mut bp_bytes2);
    let json_bytes2 = serde_json::to_vec(&db).unwrap();

    group.throughput(Throughput::Bytes(bp_bytes2.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("binproto", "DatabaseResponse"),
        &bp_bytes2,
        |b, bytes| b.iter(|| DatabaseResponse::decode(bytes).unwrap()),
    );

    group.throughput(Throughput::Bytes(json_bytes2.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("json", "DatabaseResponse"),
        &json_bytes2,
        |b, bytes| b.iter(|| serde_json::from_slice::<DatabaseResponse>(bytes).unwrap()),
    );

    group.finish();
}

// =====================================================================
// GROUPE 3 : Round-trip (encode + decode) — latence totale
// =====================================================================

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    let sensor = sample_sensor();

    group.bench_function("binproto/SensorReading", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(32);
            sensor.encode(&mut buf);
            SensorReading::decode(&buf).unwrap()
        })
    });

    group.bench_function("json/SensorReading", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(&sensor).unwrap();
            serde_json::from_slice::<SensorReading>(&bytes).unwrap()
        })
    });

    group.finish();
}

// =====================================================================
// GROUPE 4 : Comparaison de taille
// =====================================================================

fn bench_size_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("size_bytes");

    let sensor = sample_sensor();
    let mut bp_buf = Vec::new();
    sensor.encode(&mut bp_buf);
    let json_buf = serde_json::to_vec(&sensor).unwrap();

    group.throughput(Throughput::Bytes(bp_buf.len() as u64));
    group.bench_function("binproto_size/SensorReading", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            sensor.encode(&mut buf);
            buf.len()
        })
    });

    group.throughput(Throughput::Bytes(json_buf.len() as u64));
    group.bench_function("json_size/SensorReading", |b| {
        b.iter(|| serde_json::to_vec(&sensor).unwrap().len())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_roundtrip,
    bench_size_comparison
);
criterion_main!(benches);
