
use artnet_protocol::{
    ArtCommand,
    Output,
};

use std::net::{UdpSocket, Ipv4Addr};

pub struct ArtNetSender {
    socket: UdpSocket,
    target_addr: String,
}

impl ArtNetSender {
    pub fn send_dmx_frame(&self, dmx_channels: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        send_dmx_frame(&self.socket, &self.target_addr, dmx_channels)
    }
}

fn send_dmx_frame(socket: &UdpSocket, target_addr: &str, dmx_channels: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // 1. DMX-Daten als Output-Paket definieren (Universum 0)
    let output = Output {
        port_address: 0.into(), // Universum (0 bis 32767 je nach Art-Net Konfiguration)
        data: dmx_channels.to_vec().into(), // Konvertiert dein Slice in einen Bytestream (bis zu 512 Bytes)
        ..Default::default()
    };

    // 2. In den Art-Command Befehl verpacken
    let command = ArtCommand::Output(output);

    // 3. In ein Byte-Buffer für UDP serialisieren
    let bytes = command.write_to_buffer()?;

    // 4. Per UDP an den Controller senden
    socket.send_to(&bytes, target_addr)?;

    Ok(())
}

pub fn init_artnet() -> ArtNetSender {
    // --- Konfiguration ---
    // Passe diese IP-Adresse an die feste IP deines ESP32 (LuxDMX) im WLAN an:
    let esp32_ip = Ipv4Addr::new(192, 168, 2, 89); 
    let artnet_port = 6454;
    let target_addr = format!("{}:{}", esp32_ip, artnet_port);

    // UDP-Socket für den Versand der Art-Net Pakete initialisieren
    // Bind an Port 0, da wir nur senden müssen
    let socket = UdpSocket::bind("0.0.0.0:0")
        .expect("Fehler beim Binden des UDP-Sockets");
    socket.set_broadcast(true).expect("Fehler beim Setzen der Broadcast-Option");
    println!("Art-Net Sender bereit. Ziel: {}", target_addr);

    send_dmx_frame(&socket, &target_addr, &[0; 8]).expect("Fehler beim Senden des DMX-Frames");

    ArtNetSender { socket, target_addr }
}