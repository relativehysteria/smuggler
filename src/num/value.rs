use std::str::FromStr;
use crate::{Result, Error};
use crate::num::*;

/// Different values
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

impl Value {
    /// Create a new (zero) value with the format specified by `chr`
    pub fn default_from_letter(chr: char) -> Value {
        // Get the value
        match chr {
            'f' => Value::F32(0.),
            'F' => Value::F64(0.),
            'b' => Value::U8(0),
            'w' => Value::U16(0),
            'd' => Value::U32(0),
            'q' => Value::U64(0),
            'B' => Value::I8(0),
            'W' => Value::I16(0),
            'D' => Value::I32(0),
            'Q' => Value::I64(0),
            _ => unreachable!(),
        }
    }

    /// Interpret the value as a `u64` through casting
    pub fn as_u64(&self) -> u64 {
        match self {
            Self::F32(x) => x.to_bits() as u64,
            Self::F64(x) => x.to_bits(),
            Self::U8 (x) => *x as u64,
            Self::U16(x) => *x as u64,
            Self::U32(x) => *x as u64,
            Self::U64(x) => *x,
            Self::I8 (x) => *x as u64,
            Self::I16(x) => *x as u64,
            Self::I32(x) => *x as u64,
            Self::I64(x) => *x as u64,
        }
    }

    /// Get number of bytes per `self`
    pub fn bytes(&self) -> usize {
        match self {
            Self::F32(_) => 4,
            Self::F64(_) => 8,
            Self::U8 (_) => 1,
            Self::U16(_) => 2,
            Self::U32(_) => 4,
            Self::U64(_) => 8,
            Self::I8 (_) => 1,
            Self::I16(_) => 2,
            Self::I32(_) => 4,
            Self::I64(_) => 8,
        }
    }

    /// Get number of bytes per `self` when `Display`ed
    pub fn display(&self) -> usize {
        match self {
            Self::F32(_) => 25,
            Self::F64(_) => 25,
            Self::U8 (_) => 2,
            Self::U16(_) => 4,
            Self::U32(_) => 8,
            Self::U64(_) => 16,
            Self::I8 (_) => 4,
            Self::I16(_) => 6,
            Self::I32(_) => 11,
            Self::I64(_) => 21,
        }
    }

    /// Update value from little-endian bytes
    pub fn from_le_bytes(&mut self, bytes: &[u8]) {
        match self {
            Self::F32(val) =>
                *val = f32::from_le_bytes(bytes.try_into().unwrap()),
            Self::F64(val) =>
                *val = f64::from_le_bytes(bytes.try_into().unwrap()),
            Self::U8 (val) =>
                *val = u8::from_le_bytes(bytes.try_into().unwrap()),
            Self::U16(val) =>
                *val = u16::from_le_bytes(bytes.try_into().unwrap()),
            Self::U32(val) =>
                *val = u32::from_le_bytes(bytes.try_into().unwrap()),
            Self::U64(val) =>
                *val = u64::from_le_bytes(bytes.try_into().unwrap()),
            Self::I8 (val) =>
                *val = i8::from_le_bytes(bytes.try_into().unwrap()),
            Self::I16(val) =>
                *val = i16::from_le_bytes(bytes.try_into().unwrap()),
            Self::I32(val) =>
                *val = i32::from_le_bytes(bytes.try_into().unwrap()),
            Self::I64(val) =>
                *val = i64::from_le_bytes(bytes.try_into().unwrap()),
        }
    }

    /// Update `self` to a new value of the same type from `s`
    pub fn update_str(&mut self, s: &str) -> Result<()> {
        match self {
            Self::F32(val) => {
                *val = f32::from_str(s)
                    .map_err(crate::num::Error::ParseFloat)
                    .map_err(Error::Num)?;
            }
            Self::F64(val) => {
                *val = f64::from_str(s)
                    .map_err(crate::num::Error::ParseFloat)
                    .map_err(Error::Num)?;
            }
            Self::U8 (val) => *val = parse::<u8>(s)?,
            Self::U16(val) => *val = parse::<u16>(s)?,
            Self::U32(val) => *val = parse::<u32>(s)?,
            Self::U64(val) => *val = parse::<u64>(s)?,
            Self::I8 (val) => *val = parse::<i8>(s)?,
            Self::I16(val) => *val = parse::<i16>(s)?,
            Self::I32(val) => *val = parse::<i32>(s)?,
            Self::I64(val) => *val = parse::<i64>(s)?,
        }

        Ok(())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F32(val) =>
                f.write_fmt(format_args!("{:25.6}", val)),
            Self::F64(val) =>
                f.write_fmt(format_args!("{:25.6}", val)),
            Self::U8 (val) => f.write_fmt(format_args!("{:02x}", val)),
            Self::U16(val) => f.write_fmt(format_args!("{:04x}", val)),
            Self::U32(val) => f.write_fmt(format_args!("{:08x}", val)),
            Self::U64(val) => f.write_fmt(format_args!("{:016x}", val)),
            Self::I8 (val) => f.write_fmt(format_args!("{:4}", val)),
            Self::I16(val) => f.write_fmt(format_args!("{:6}", val)),
            Self::I32(val) => f.write_fmt(format_args!("{:11}", val)),
            Self::I64(val) => f.write_fmt(format_args!("{:21}", val)),
        }
    }
}

impl std::fmt::LowerHex for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F32(val) =>
                f.write_fmt(format_args!("{:08x}", val.to_bits())),
            Self::F64(val) =>
                f.write_fmt(format_args!("{:016x}", val.to_bits())),
            Self::U8 (val) => f.write_fmt(format_args!("{:02x}", val)),
            Self::U16(val) => f.write_fmt(format_args!("{:04x}", val)),
            Self::U32(val) => f.write_fmt(format_args!("{:08x}", val)),
            Self::U64(val) => f.write_fmt(format_args!("{:016x}", val)),
            Self::I8 (val) => f.write_fmt(format_args!("{:02x}", val)),
            Self::I16(val) => f.write_fmt(format_args!("{:04x}", val)),
            Self::I32(val) => f.write_fmt(format_args!("{:08x}", val)),
            Self::I64(val) => f.write_fmt(format_args!("{:016x}", val)),
        }
    }
}
