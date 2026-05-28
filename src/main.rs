use pcap::{Capture, Device};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

// Import our new photon module
mod photon;
use photon::{parse_photon_header, parse_command_header};

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
    println!("Sniffer started. Launch Albion Online...");

    while let Ok(packet) = cap.next_packet() {
        if let Some(ethernet) = EthernetPacket::new(packet.data) {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                    let payload = udp.payload();

                    // If the payload is too small to be a Photon packet, skip it
                    if payload.len() < 12 {
                        continue;
                    }

                    // 1. Try to parse the Photon Header
                    match parse_photon_header(payload) {
                        Ok((mut remaining_bytes, header)) => {
                            // If it's a valid packet but has 0 commands (like a ping), ignore it
                            if header.command_count == 0 {
                                continue;
                            }

                            // 2. Loop through the number of commands defined in the header
                            for _ in 0..header.command_count {
                                // Try to parse the Command Header
                                match parse_command_header(remaining_bytes) {
                                    Ok((next_bytes, cmd_header)) => {
                                        // Command Type 1 is "SendReliable" (Unfragmented data)
                                        // Command Type 8 is "SendReliableFragment" (Fragmented data)
                                        if cmd_header.command_type == 1 || cmd_header.command_type == 8 {
                                            println!(
                                                "Found Command! Type: {}, Length: {}",
                                                cmd_header.command_type, cmd_header.length
                                            );
                                        }

                                        // Move the pointer forward by the length of this command's data 
                                        // so we can parse the next command in the loop
                                        let payload_length = (cmd_header.length - 12) as usize; 
                                        if next_bytes.len() >= payload_length {
                                            remaining_bytes = &next_bytes[payload_length..];
                                        } else {
                                            break; // Packet was malformed or cut off
                                        }
                                    }
                                    Err(_) => break, // Failed to parse command header
                                }
                            }
                        }
                        Err(_) => {
                            // Not a valid Photon packet
                        }
                    }
                }
            }
        }
    }
}