// binproto/src/lib.rs
pub mod schema;
pub mod generator;
pub mod server;
pub mod client;
pub mod debugger;
pub mod multilang;

// =========================
// TRAITS + ERROR
// =========================

pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>);
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    NotEnoughData,
    InvalidData,
}

pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError>;
}

// =========================
// VARINT (LEB128)
// =========================

pub fn encode_varint(mut val: u64, buf: &mut Vec<u8>) {
    while val >= 0x80 {
        buf.push((val as u8) | 0x80);
        val >>= 7;
    }
    buf.push(val as u8);
}

pub fn decode_varint(buf: &[u8]) -> Result<(u64, usize), DecodeError> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut i = 0;

    for byte in buf {
        let b = *byte as u64;
        result |= (b & 0x7F) << shift;

        i += 1;

        if b & 0x80 == 0 {
            return Ok((result, i));
        }

        shift += 7;
        if shift > 63 {
            return Err(DecodeError::InvalidData);
        }
    }

    Err(DecodeError::NotEnoughData)
}

// =========================
// ZIGZAG (pour i32 / i64)
// =========================

fn zigzag_encode_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn zigzag_decode_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

fn zigzag_encode_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

fn zigzag_decode_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

// =========================
// IMPL PRIMITIFS
// =========================

// u8
impl Encode for u8 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self);
    }
}

impl Decode for u8 {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buf.is_empty() {
            return Err(DecodeError::NotEnoughData);
        }
        Ok((buf[0], 1))
    }
}

// u32
impl Encode for u32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        encode_varint(*self as u64, buf);
    }
}

impl Decode for u32 {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (v, size) = decode_varint(buf)?;
        Ok((v as u32, size))
    }
}

// u64
impl Encode for u64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        encode_varint(*self, buf);
    }
}

impl Decode for u64 {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        decode_varint(buf)
    }
}

// i32
impl Encode for i32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        encode_varint(zigzag_encode_i32(*self) as u64, buf);
    }
}

impl Decode for i32 {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (v, size) = decode_varint(buf)?;
        Ok((zigzag_decode_i32(v as u32), size))
    }
}

// i64
impl Encode for i64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        encode_varint(zigzag_encode_i64(*self), buf);
    }
}

impl Decode for i64 {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (v, size) = decode_varint(buf)?;
        Ok((zigzag_decode_i64(v), size))
    }
}

// bool
impl Encode for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(if *self { 1 } else { 0 });
    }
}

impl Decode for bool {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buf.is_empty() {
            return Err(DecodeError::NotEnoughData);
        }
        match buf[0] {
            0 => Ok((false, 1)),
            1 => Ok((true, 1)),
            _ => Err(DecodeError::InvalidData),
        }
    }
}

// String
impl Encode for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        encode_varint(bytes.len() as u64, buf);
        buf.extend_from_slice(bytes);
    }
}

impl Decode for String {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (len, offset) = decode_varint(buf)?;
        let len = len as usize;

        if buf.len() < offset + len {
            return Err(DecodeError::NotEnoughData);
        }

        let s = std::str::from_utf8(&buf[offset..offset + len])
            .map_err(|_| DecodeError::InvalidData)?;

        Ok((s.to_string(), offset + len))
    }
}

// Vec<T>
impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, buf: &mut Vec<u8>) {
        encode_varint(self.len() as u64, buf);
        for item in self {
            item.encode(buf);
        }
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (len, mut offset) = decode_varint(buf)?;
        let len = len as usize;

        let mut items = Vec::with_capacity(len);

        for _ in 0..len {
            let (val, size) = T::decode(&buf[offset..])?;
            items.push(val);
            offset += size;
        }

        Ok((items, offset))
    }
}

// =========================
// TESTS
// =========================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint() {
        let mut buf = Vec::new();
        encode_varint(300, &mut buf);
        assert_eq!(buf, vec![0xAC, 0x02]);

        let (val, size) = decode_varint(&buf).unwrap();
        assert_eq!(val, 300);
        assert_eq!(size, 2);
    }

    #[test]
    fn test_u32() {
        let val = 150u32;
        let mut buf = Vec::new();
        val.encode(&mut buf);

        let (decoded, _) = u32::decode(&buf).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn test_i32_negative() {
        let val = -42i32;
        let mut buf = Vec::new();
        val.encode(&mut buf);

        let (decoded, _) = i32::decode(&buf).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn test_string() {
        let val = String::from("hello");
        let mut buf = Vec::new();
        val.encode(&mut buf);

        let (decoded, _) = String::decode(&buf).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn test_empty_string() {
        let val = String::new();
        let mut buf = Vec::new();
        val.encode(&mut buf);

        let (decoded, _) = String::decode(&buf).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn test_vec_u8() {
        let val = vec![1u8, 2, 3];
        let mut buf = Vec::new();
        val.encode(&mut buf);

        let (decoded, _) = Vec::<u8>::decode(&buf).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn test_vec_generic() {
        let val = vec![1u32, 2, 300];
        let mut buf = Vec::new();
        val.encode(&mut buf);

        let (decoded, _) = Vec::<u32>::decode(&buf).unwrap();
        assert_eq!(val, decoded);
    }
}