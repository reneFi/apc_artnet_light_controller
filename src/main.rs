mod output;
mod midi;
mod control;

use crate::output::init_artnet;
use crate::midi::midi_init;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== APCmini MK2 to LuxDMX (Art-Net) ===");
    
    let artnet_sender = init_artnet();
    let mut midi_controller = midi_init()?;
    
    midi_controller.update_button_leds(&[0, 1, 2, 3], &[])?;
    
    control::run(&midi_controller, &artnet_sender)


}
