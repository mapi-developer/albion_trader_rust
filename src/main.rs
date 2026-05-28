use pcap::{Device, Capture};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

fn main() {
    // 1. Find the default network device (like eth0 or wlan0)
    let main_device = Device::lookup()
        .expect("Failed to lookup device")
        .expect("No devices found");
    
    println!("Listening on device: {}", main_device.name);

    // 2. Open the capture handle
    let mut cap = Capture::from_device(main_device)
        .unwrap()
        .promisc(true)     // Listen to all traffic on the interface
        .snaplen(65535)    // Capture the whole packet
        .timeout(100)      // 100ms read timeout
        .open()
        .unwrap();

    // 3. Apply the BPF filter (same as your Python script)
    cap.filter("udp portrange 5055-5056", true).unwrap();

    println!("Sniffer started. Launch Albion Online...");

    // 4. Capture loop
    while let Ok(packet) = cap.next_packet() {
        // Parse the raw bytes into an Ethernet packet
        if let Some(ethernet) = EthernetPacket::new(packet.data) {
            
            // Parse into IPv4
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                
                // Parse into UDP
                if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                    
                    let payload = udp.payload();
                    
                    // We only care about packets that actually contain data
                    if !payload.is_empty() {
                        println!(
                            "Captured Photon Packet | Size: {} bytes | {} -> {}", 
                            payload.len(),
                            udp.get_source(),
                            udp.get_destination()
                        );
                        
                        // TODO: Pass `payload` to our Photon decoder
                    }
                }
            }
        }
    }
}