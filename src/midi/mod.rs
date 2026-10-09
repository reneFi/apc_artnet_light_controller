use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort,
};
use std::error::Error;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, RecvError};
use std::thread;
use std::time::Duration;

const BUTTON_COLOR_GREEN: u8 = 5;
const BUTTON_COLOR_RED: u8 = 13;

#[derive(Debug, PartialEq, Eq)]
pub enum MidiEvent {
    Button { note: u8, pressed: bool },
    Fader { index: u8, value: u8 },
}

pub struct MidiController {
    events: Receiver<MidiEvent>,
    _connection: MidiInputConnection<()>,
    output: MidiOutputConnection,
}

impl MidiController {
    pub fn recv(&self) -> Result<MidiEvent, RecvError> {
        self.events.recv()
    }

    /// Sets used buttons to green and the activated subset to red.
    pub fn update_button_leds(
        &mut self,
        used_buttons: &[u8],
        activated_buttons: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        for &button in used_buttons {
            if button > 127 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "MIDI button note must be in 0..=127",
                )
                .into());
            }

            self.output.send(&button_led_message(
                button,
                activated_buttons.contains(&button),
            ))?;
        }

        Ok(())
    }
}

fn button_led_message(button: u8, activated: bool) -> [u8; 3] {
    [
        0x90,
        button,
        if activated {
            BUTTON_COLOR_RED
        } else {
            BUTTON_COLOR_GREEN
        },
    ]
}

fn parse_event(message: &[u8]) -> Option<MidiEvent> {
    if message.len() < 3 {
        return None;
    }

    let message_type = message[0] & 0xF0;
    let control = message[1];
    let value = message[2];

    match message_type {
        0x80 => Some(MidiEvent::Button {
            note: control,
            pressed: false,
        }),
        0x90 => Some(MidiEvent::Button {
            note: control,
            pressed: value > 0,
        }),
        0xB0 if (48..=56).contains(&control) => Some(MidiEvent::Fader {
            index: control - 48,
            value,
        }),
        _ => None,
    }
}

pub fn midi_init() -> Result<MidiController, Box<dyn std::error::Error>> {
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

    println!("Lausche auf MIDI-Events (Drücke Pads oder Fader). Drücke Strg+C zum Beenden...\n");

    // Eingang verbinden und Event-Callback definieren
    let (event_tx, event_rx) = mpsc::channel();
    let connection = midi_in.connect(
        &in_port,
        "apc_test_input",
        move |_stamp, message, _| {
            if let Some(event) = parse_event(message) {
                let _ = event_tx.send(event);
            }
        },
        (),
    )?;
    Ok(MidiController {
        events: event_rx,
        _connection: connection,
        output: conn_out,
    })
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

#[cfg(test)]
mod tests {
    use super::{button_led_message, parse_event, MidiEvent};

    #[test]
    fn maps_button_led_state_to_akai_colors() {
        assert_eq!(button_led_message(7, false), [0x90, 7, 5]);
        assert_eq!(button_led_message(7, true), [0x90, 7, 13]);
    }

    #[test]
    fn parses_button_press_and_release() {
        assert_eq!(
            parse_event(&[0x91, 7, 127]),
            Some(MidiEvent::Button {
                note: 7,
                pressed: true
            })
        );
        assert_eq!(
            parse_event(&[0x81, 7, 0]),
            Some(MidiEvent::Button {
                note: 7,
                pressed: false
            })
        );
        assert_eq!(
            parse_event(&[0x91, 7, 0]),
            Some(MidiEvent::Button {
                note: 7,
                pressed: false
            })
        );
    }

    #[test]
    fn parses_fader_index_and_value() {
        assert_eq!(
            parse_event(&[0xB0, 48, 64]),
            Some(MidiEvent::Fader { index: 0, value: 64 })
        );
        assert_eq!(
            parse_event(&[0xB0, 56, 127]),
            Some(MidiEvent::Fader {
                index: 8,
                value: 127
            })
        );
    }

    #[test]
    fn ignores_unhandled_and_incomplete_messages() {
        assert_eq!(parse_event(&[0xB0, 47, 64]), None);
        assert_eq!(parse_event(&[0xC0, 48, 64]), None);
        assert_eq!(parse_event(&[0x90, 7]), None);
    }
}