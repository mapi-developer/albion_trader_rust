use std::collections::HashMap;
use nom::{
    number::complete::{be_f32, be_u16, be_u32, be_u64, be_u8},
    bytes::complete::take,
    IResult,
};

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
    Dictionary(Vec<(PhotonValue, PhotonValue)>),
    Hashtable(Vec<(PhotonValue, PhotonValue)>),
}

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
            if len == 0 {
                return Ok((input, PhotonValue::Slice(Vec::new())));
            }
            let (input, array_type) = be_u8(input)?;
            let mut current_input = input;
            let mut arr = Vec::new();
            for _ in 0..len {
                if current_input.is_empty() { break; }
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
                if current_input.len() < 2 { break; }
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
                if current_input.len() < 2 { break; }
                let (next_input, k_type) = be_u8(current_input)?;
                let (next_input, k) = parse_value(next_input, k_type)?;
                let (next_input, v_type) = be_u8(next_input)?;
                let (next_input, v) = parse_value(next_input, v_type)?;
                hash.push((k, v));
                current_input = next_input;
            }
            Ok((current_input, PhotonValue::Hashtable(hash)))
        },
        unknown_type => {
            // Instead of crashing, we gracefully fail but print what type code broke us
            println!("[Decoder] Unknown parameter type code found: {}", unknown_type);
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
        }
    }
}

pub fn parse_reliable_message(input: &[u8]) -> Result<HashMap<u8, PhotonValue>, String> {
    if input.len() < 2 { return Err("Payload too small".to_string()); }
    
    let (input, _signature) = be_u8::<_, nom::error::Error<&[u8]>>(input).map_err(|e| e.to_string())?;
    let (input, message_type) = be_u8(input).map_err(|e| e.to_string())?;

    // Safe parsing bounds checking for type parameters
    let input = match message_type {
        2 | 4 => {
            if input.is_empty() { return Err("Truncated message".to_string()); }
            &input[1..] // Skip operation_code/event_code byte (we look up parameters directly)
        },
        3 | 7 => {
            if input.len() < 4 { return Err("Truncated response".to_string()); }
            &input[4..] // Skip response headers
        },
        _ => input
    };

    if input.len() < 2 { return Err("Missing parameter count".to_string()); }
    let (mut current_input, param_count) = be_u16::<_, nom::error::Error<&[u8]>>(input).map_err(|e| e.to_string())?;

    let mut parameters = HashMap::new();

    for _ in 0..param_count {
        if current_input.len() < 2 { break; }
        
        let (next_input, param_id) = be_u8(current_input).map_err(|e| e.to_string())?;
        let (next_input, param_type) = be_u8(next_input).map_err(|e| e.to_string())?;
        
        match parse_value(next_input, param_type) {
            Ok((rem, value)) => {
                parameters.insert(param_id, value);
                current_input = rem;
            },
            Err(_) => {
                // Stop processing parameters if one field fails, but return what we have!
                break;
            }
        }
    }

    Ok(parameters)
}