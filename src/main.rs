mod output;
mod midi;

use crate::output::init_artnet;
use crate::midi::midi_init;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    init_artnet();
    let mut midi_controller = midi_init()?;
    
    midi_controller.update_button_leds(&[0, 1, 2, 3], &[])?;
    
    loop {
        let event = midi_controller.recv()?;
        println!("APC mini event: {:?}", event);
    }

/*
    // 1. MIDI Input initialisieren
    let midi_in = MidiInput::new("apc_test_in")?;
    let in_ports = midi_in.ports();
    
    // 2. MIDI Output initialisieren
    let midi_out = MidiOutput::new("apc_test_out")?;
    let out_ports = midi_out.ports();

    // Ports anhand des Namens ("APCmini mk2") suchen oder interaktiv wählen
    let in_port = select_in_port(&midi_in, &in_ports, "APC mini mk2:APC mini mk2 Control")?;
    let out_port = select_out_port(&midi_out, &out_ports, "APC mini mk2:APC mini mk2 Control")?;

    println!("APCmini MK2 erfolgreich gefunden und verbunden!");

    let mut conn_out = midi_out.connect(&out_port, "apc_test_output")?;
    
    // 1. WICHTIG: Korrekter SysEx-Befehl für den APCmini MK2 "Application/Programmer Mode"
    // Schaltet das Gerät in den steuerbaren Modus abseits von Ableton Live
    let sysex_app_mode: &[u8] = &[0xF0, 0x47, 0x7F, 0x43, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF7];
    conn_out.send(sysex_app_mode)?;
    
    // Kurze Pause, damit das Gerät den Modus übernimmt
    thread::sleep(Duration::from_millis(50));

    // 2. LED ansteuern:
    // Nach Protokoll steuert der MIDI-Kanal im Status-Byte das Verhalten:
    // 0x90 (Kanal 1) = Statisch 100% Helligkeit
    // Byte 2 = Notennummer (0 bis 63 für das 8x8 Pad-Grid, 0 ist unten links / je nach Mapping)
    // Byte 3 = Velocity (Farbcode aus der Akai RGB-Palette, z. B. 5 = Grün, 13 = Rot, 45 = Blau)
    
    println!("Schalte Test-LEDs ein...");
    
    // Test: Lasse die ersten 4 Pads in unterschiedlichen Farben aufleuchten
    conn_out.send(&[0x90, 0, 13])?; // Pad 0: Rot (Velocity 13)
    conn_out.send(&[0x90, 1, 5])?;  // Pad 1: Grün (Velocity 5)
    conn_out.send(&[0x90, 2, 45])?; // Pad 2: Blau (Velocity 45)
    conn_out.send(&[0x90, 3, 49])?; // Pad 3: Weiß/Cyan (Velocity 49)

     // Kurze Pause, damit das Gerät den Modus übernimmt
    thread::sleep(Duration::from_millis(2000));

    // Test: Lasse die ersten 4 Pads in unterschiedlichen Farben aufleuchten
    conn_out.send(&[0x90, 0, 0])?; // Pad 0: Aus
    conn_out.send(&[0x90, 1, 0])?;  // Pad 1: Aus
    conn_out.send(&[0x90, 2, 0])?; // Pad 2: Aus
    conn_out.send(&[0x90, 3, 0])?; // Pad 3: Aus

    println!("Lausche auf MIDI-Events (Drücke Pads oder Fader). Drücke Strg+C zum Beenden...\n");

    // Eingang verbinden und Event-Callback definieren
    let _conn_in = midi_in.connect(
        &in_port,
        "apc_test_input",
        move |_stamp, message, _| {
            if message.len() >= 3 {
                let status = message[0];
                let note = message[1];
                let velocity = message[2];
                let mut forwarded = message.to_vec();
                forwarded[2] = 45;
                conn_out.send(&forwarded); // Leite das Event direkt an den Output weiter
                println!("MIDI Event -> Status: {:#04X}, Note/Control: {}, Velocity/Value: {}", status, note, velocity);
            } else {
                println!("SysEx / Anderes Event: {:02X?}", message);
            }
        },
        (),
    )?;
*/
}
/*
// Hilfsfunktion für Input-Port Auswahl
fn select_in_port(
    midi_in: &MidiInput,
    ports: &[MidiInputPort],
    search_str: &str,
) -> Result<MidiInputPort, Box<dyn Error>> {
    if ports.is_empty() {
        return Err("Keine MIDI-Eingabeports gefunden. Ist die APCmini angeschlossen?".into());
    }

    // Automatische Suche nach dem Namen
    for port in ports {
        let name = midi_in.port_name(port)?;
        if name.to_lowercase().contains(&search_str.to_lowercase()) {
            println!("Automatisch gefunden (In): {}", name);
            return Ok(port.clone());
        }
    }

    // Fallback: Interaktive Liste im Terminal
    println!("Kein Port mit '{}' im Namen gefunden. Verfügbare Eingaben:", search_str);
    for (i, p) in ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
    }
    print!("Wähle Port-Nummer: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let index: usize = input.trim().parse()?;

    ports.get(index)
        .cloned()
        .ok_or_else(|| "Ungültige Port-Nummer".into())
}

// Hilfsfunktion für Output-Port Auswahl
fn select_out_port(
    midi_out: &MidiOutput,
    ports: &[MidiOutputPort],
    search_str: &str,
) -> Result<MidiOutputPort, Box<dyn Error>> {
    if ports.is_empty() {
        return Err("Keine MIDI-Ausgabeports gefunden. Ist die APCmini angeschlossen?".into());
    }

    // Automatische Suche nach dem Namen
    for port in ports {
        let name = midi_out.port_name(port)?;
        if name.to_lowercase().contains(&search_str.to_lowercase()) {
            println!("Automatisch gefunden (Out): {}", name);
            return Ok(port.clone());
        }
    }

    // Fallback: Interaktive Liste im Terminal
    println!("Kein Port mit '{}' im Namen gefunden. Verfügbare Ausgaben:", search_str);
    for (i, p) in ports.iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
    }
    print!("Wähle Port-Nummer: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let index: usize = input.trim().parse()?;

    ports.get(index)
        .cloned()
        .ok_or_else(|| "Ungültige Port-Nummer".into())
}
*/