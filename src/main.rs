use midir::{MidiInput, MidiOutput, MidiInputPort, MidiOutputPort};
use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== APCmini MK2 MIDI Test (midir 0.11) ===");

    // 1. MIDI Input initialisieren
    let midi_in = MidiInput::new("apc_test_in")?;
    let in_ports = midi_in.ports();
    
    // 2. MIDI Output initialisieren
    let midi_out = MidiOutput::new("apc_test_out")?;
    let out_ports = midi_out.ports();

    // Ports anhand des Namens ("APCmini mk2") suchen oder interaktiv wählen
    let in_port = select_in_port(&midi_in, &in_ports, "APCmini mk2")?;
    let out_port = select_out_port(&midi_out, &out_ports, "APCmini mk2")?;

    println!("APCmini MK2 erfolgreich gefunden und verbunden!");

    // Ausgangsverbindung öffnen und APCmini MK2 in den programmierbaren Modus versetzen (SysEx)
    let mut conn_out = midi_out.connect(&out_port, "apc_test_output")?;
    
    // APCmini MK2 DAW / Standalone Initialization SysEx 
    let sysex_init: &[u8] = &[0xF0, 0x47, 0x7F, 0x43, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF7];
    conn_out.send(sysex_init)?;
    
    // Optisches Feedback: Setze Pad (Row 0, Col 0 -> Note 0) auf Hellgrün
    conn_out.send(&[0x90, 0, 5])?;

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
                println!("MIDI Event -> Status: {:#04X}, Note/Control: {}, Velocity/Value: {}", status, note, velocity);
            } else {
                println!("SysEx / Anderes Event: {:02X?}", message);
            }
        },
        (),
    )?;

    // Hauptschleife am Leben halten
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

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