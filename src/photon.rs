use nom::{
    number::complete::{be_u16, be_u32, be_u8},
    bytes::complete::take,
    IResult,
};

// 1. Define the structure of the main Photon packet header
#[derive(Debug)]
pub struct PhotonHeader {
    pub peer_id: u16,
    pub crc_enabled: u8,
    pub command_count: u8,
    pub timestamp: u32,
    pub challenge: u32,
}

// 2. Define the structure of a Command inside the packet
#[derive(Debug)]
pub struct CommandHeader {
    pub command_type: u8,
    pub channel_id: u8,
    pub command_flags: u8,
    pub reserved_byte: u8,
    pub length: u32,
    pub reliable_sequence_number: u32,
}

// 3. Write a parser for the Photon Header
pub fn parse_photon_header(input: &[u8]) -> IResult<&[u8], PhotonHeader> {
    // Network data is Big Endian (be_). We parse bytes sequentially.
    let (input, peer_id) = be_u16(input)?;
    let (input, crc_enabled) = be_u8(input)?;
    let (input, command_count) = be_u8(input)?;
    let (input, timestamp) = be_u32(input)?;
    let (input, challenge) = be_u32(input)?;

    Ok((
        input, // Return the remaining unparsed bytes
        PhotonHeader {
            peer_id,
            crc_enabled,
            command_count,
            timestamp,
            challenge,
        },
    ))
}

// 4. Write a parser for the Command Header
pub fn parse_command_header(input: &[u8]) -> IResult<&[u8], CommandHeader> {
    let (input, command_type) = be_u8(input)?;
    let (input, channel_id) = be_u8(input)?;
    let (input, command_flags) = be_u8(input)?;
    let (input, reserved_byte) = be_u8(input)?;
    let (input, length) = be_u32(input)?;
    let (input, reliable_sequence_number) = be_u32(input)?;

    Ok((
        input,
        CommandHeader {
            command_type,
            channel_id,
            command_flags,
            reserved_byte,
            length,
            reliable_sequence_number,
        },
    ))
}