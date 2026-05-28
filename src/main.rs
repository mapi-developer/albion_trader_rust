use pcap::{Capture, Device};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::io::Read;
use flate2::read::GzDecoder;

// Tell Rust to use the modules we created
mod photon;
mod fragment_buffer;
mod photon_decode;
mod constants;
mod handlers;

use photon::{parse_photon_header, parse_command_header, parse_fragment_header};
use fragment_buffer::FragmentBuffer;
use photon_decode::parse_reliable_message;
use handlers::route_reliable_message;

fn main() {
    let main_device = Device::lookup()
        .expect("Failed to lookup device")
        .expect("No devices found");

    println!("Listening on device: {}", main_device.name);

    let mut cap = Capture::from_device(main_device)
        .unwrap()
        .promisc(true)
        .snaplen(65535)
        .timeout(100)
        .open()
        .unwrap();

    cap.filter("udp portrange 5055-5056", true).unwrap();
    println!("Sniffer started. Launch Albion Online and open the Market...");

    // Initialize our memory-safe Fragment Buffer
    let mut frag_buffer = FragmentBuffer::new();

    while let Ok(packet) = cap.next_packet() {
        if let Some(ethernet) = EthernetPacket::new(packet.data) {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                    let payload = udp.payload();

                    if payload.len() < 12 { continue; }

                    // 1. Parse the main Photon Header
                    if let Ok((mut remaining_bytes, header)) = parse_photon_header(payload) {
                        if header.command_count == 0 { continue; }

                        // 2. Loop through all commands in the packet
                        for _ in 0..header.command_count {
                            if let Ok((next_bytes, cmd_header)) = parse_command_header(remaining_bytes) {
                                let payload_length = (cmd_header.length - 12) as usize;
                                
                                if next_bytes.len() < payload_length {
                                    break; 
                                }

                                // This is the actual data for this specific command
                                let command_data = &next_bytes[..payload_length];

                                // Command Type 1: SendReliable (Unfragmented data, like moving or silver updates)
                                if cmd_header.command_type == 1 {
                                    handle_payload(command_data);
                                } 
                                // Command Type 8: SendReliableFragment (Big data, like Market Orders)
                                else if cmd_header.command_type == 8 {
                                    
                                    // Parse the extra Fragment Header
                                    if let Ok((frag_data, frag_header)) = parse_fragment_header(command_data) {
                                        
                                        // Feed it to our Buffer. If it returns Some(data), the packet is fully reassembled!
                                        if let Some(assembled_payload) = frag_buffer.offer(
                                            frag_header.start_sequence_number,
                                            frag_header.fragment_count,
                                            frag_header.total_length,
                                            frag_header.fragment_offset,
                                            frag_data
                                        ) {
                                            println!("--- Successfully Assembled {} Fragments! ---", frag_header.fragment_count);
                                            handle_payload(&assembled_payload);
                                        }
                                    }
                                }

                                // Move the pointer forward for the next command in the loop
                                remaining_bytes = &next_bytes[payload_length..];
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

// 3. This function processes fully assembled payloads
fn handle_payload(payload: &[u8]) {
    let mut gz = GzDecoder::new(payload);
    let mut decompressed = Vec::new();
    
    let process_data = match gz.read_to_end(&mut decompressed) {
        Ok(_) => decompressed.as_slice(),
        Err(_) => payload,
    };

    if let Ok((_, reliable_msg)) = parse_reliable_message(process_data) {
        // Route the fully decoded message to our handler structure
        route_reliable_message(
            reliable_msg.message_type,
            reliable_msg.code,
            &reliable_msg.parameters,
        );
    }
}