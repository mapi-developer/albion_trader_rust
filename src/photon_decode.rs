use std::collections::HashMap;
use nom::{
    number::complete::{be_f32, be_u16, be_u32, be_u64, be_u8},
    bytes::complete::take,
    IResult,
};

// 1. The Rust equivalent of Python's dynamic types
#[derive(Debug, Clone)]
pub enum PhotonValue {
    Nil,
    Int8(i8),
    Int16(u16),
    Int32(u32),
    Int64(u64),
    Float32(f32),
    String(String),
    Boolean(bool),
    ByteSlice(Vec<u8>),
    Slice(Vec<PhotonValue>),
    Dictionary(Vec<(PhotonValue, PhotonValue)>), // Using Vec of tuples for simple key-value storage
    Hashtable(Vec<(PhotonValue, PhotonValue)>),
}

// 2. The main payload structure
#[derive(Debug)]
pub struct ReliableMessage {
    pub signature: u8,
    pub message_type: u8,
    pub code: u8, // event_code or operation_code
    pub parameters: HashMap<u8, PhotonValue>,
}

// 3. Recursive parser for all Photon Data Types
pub fn parse_value(input: &[u8], param_type: u8) -> IResult<&[u8], PhotonValue> {
    match param_type {
        0 | 42 => Ok((input, PhotonValue::Nil)),
        98 => { // 'b' - Int8
            let (input, val) = be_u8(input)?;
            Ok((input, PhotonValue::Int8(val as i8)))
        },
        7 | 107 => { // 'k' - Int16
            let (input, val) = be_u16(input)?;
            Ok((input, PhotonValue::Int16(val)))
        },
        105 => { // 'i' - Int32
            let (input, val) = be_u32(input)?;
            Ok((input, PhotonValue::Int32(val)))
        },
        108 => { // 'l' - Int64
            let (input, val) = be_u64(input)?;
            Ok((input, PhotonValue::Int64(val)))
        },
        102 => { // 'f' - Float32
            let (input, val) = be_f32(input)?;
            Ok((input, PhotonValue::Float32(val)))
        },
        111 | 1 => { // 'o' - Boolean
            let (input, val) = be_u8(input)?;
            Ok((input, PhotonValue::Boolean(val != 0)))
        },
        115 => { // 's' - String
            let (input, len) = be_u16(input)?;
            let (input, str_bytes) = take(len)(input)?;
            let s = String::from_utf8_lossy(str_bytes).into_owned();
            Ok((input, PhotonValue::String(s)))
        },
        120 => { // 'x' - Byte Slice
            let (input, len) = be_u32(input)?;
            let (input, bytes) = take(len)(input)?;
            Ok((input, PhotonValue::ByteSlice(bytes.to_vec())))
        },
        121 => { // 'y' - Array / Slice
            let (input, len) = be_u16(input)?;
            let (input, array_type) = be_u8(input)?;
            let mut current_input = input;
            let mut arr = Vec::new();
            for _ in 0..len {
                let (next_input, val) = parse_value(current_input, array_type)?;
                arr.push(val);
                current_input = next_input;
            }
            Ok((current_input, PhotonValue::Slice(arr)))
        },
        68 => { // 'D' - Dictionary
            let (input, key_type) = be_u8(input)?;
            let (input, val_type) = be_u8(input)?;
            let (input, len) = be_u16(input)?;
            let mut current_input = input;
            let mut dict = Vec::new();
            for _ in 0..len {
                let (next_input, k) = parse_value(current_input, key_type)?;
                let (next_input, v) = parse_value(next_input, val_type)?;
                dict.push((k, v));
                current_input = next_input;
            }
            Ok((current_input, PhotonValue::Dictionary(dict)))
        },
        104 => { // 'h' - Hashtable
            let (input, len) = be_u16(input)?;
            let mut current_input = input;
            let mut hash = Vec::new();
            for _ in 0..len {
                let (next_input, k_type) = be_u8(current_input)?;
                let (next_input, k) = parse_value(next_input, k_type)?;
                let (next_input, v_type) = be_u8(next_input)?;
                let (next_input, v) = parse_value(next_input, v_type)?;
                hash.push((k, v));
                current_input = next_input;
            }
            Ok((current_input, PhotonValue::Hashtable(hash)))
        },
        _ => {
            // Unrecognized type - return an error so the parser safely bails out
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
        }
    }
}

// 4. Parser for the overall Reliable Message
pub fn parse_reliable_message(input: &[u8]) -> IResult<&[u8], ReliableMessage> {
    let (input, signature) = be_u8(input)?;
    let (input, message_type) = be_u8(input)?;

    // Mimicking Python's `read_data`: EventData (4) and OperationRequest (2) read 1 byte.
    // OperationResponse (3 or 7) reads 4 bytes.
    let (input, code) = match message_type {
        2 | 4 => be_u8(input)?,
        3 | 7 => {
            let (rem, op_code) = be_u8(input)?;
            let (rem, _) = take(3usize)(rem)?; // Skip response_code + debug type for now
            (rem, op_code)
        },
        _ => (input, 0)
    };

    let (input, param_count) = be_u16(input)?;

    let mut current_input = input;
    let mut parameters = HashMap::new();

    for _ in 0..param_count {
        // Stop if we don't have enough bytes to read id + type
        if current_input.len() < 2 { break; }
        
        let (next_input, param_id) = be_u8(current_input)?;
        let (next_input, param_type) = be_u8(next_input)?;
        
        match parse_value(next_input, param_type) {
            Ok((rem, value)) => {
                parameters.insert(param_id, value);
                current_input = rem;
            },
            Err(_) => break, // Safely handle unknown formats
        }
    }

    Ok((current_input, ReliableMessage {
        signature,
        message_type,
        code,
        parameters
    }))
}