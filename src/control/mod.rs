use crate::midi::{MidiController, MidiEvent};
use crate::output::ArtNetSender;
use std::error::Error;

const DMX_CHANNEL_COUNT: usize = 8;
const MIDI_MAX_VALUE: u16 = 127;
const DMX_MAX_VALUE: u16 = 255;

struct ControlState {
	channels: [u8; DMX_CHANNEL_COUNT],
	master: u8,
}

impl Default for ControlState {
	fn default() -> Self {
		Self {
			channels: [0; DMX_CHANNEL_COUNT],
			master: 127,
		}
	}
}

impl ControlState {
	fn apply_fader(&mut self, index: u8, value: u8) -> Option<[u8; DMX_CHANNEL_COUNT]> {
		match index as usize {
			channel if channel < DMX_CHANNEL_COUNT => self.channels[channel] = value,
			DMX_CHANNEL_COUNT => self.master = value,
			_ => return None,
		}

		Some(std::array::from_fn(|channel| {
			let level = u16::from(self.channels[channel]) * DMX_MAX_VALUE / MIDI_MAX_VALUE;
			(level * u16::from(self.master) / MIDI_MAX_VALUE) as u8
		}))
	}
}

pub fn run(
	midi_controller: &MidiController,
	artnet_sender: &ArtNetSender,
) -> Result<(), Box<dyn Error>> {
	let mut state = ControlState::default();

	loop {
		if let MidiEvent::Fader { index, value } = midi_controller.recv()? {
			if let Some(frame) = state.apply_fader(index, value) {
				artnet_sender.send_dmx_frame(&frame)?;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::ControlState;

	#[test]
	fn maps_faders_to_matching_dmx_channels() {
		let mut state = ControlState::default();

		let frame = state.apply_fader(0, 127).unwrap();
		assert_eq!(frame, [255, 0, 0, 0, 0, 0, 0, 0]);

		let frame = state.apply_fader(1, 64).unwrap();
		assert_eq!(frame, [255, 128, 0, 0, 0, 0, 0, 0]);

		let frame = state.apply_fader(7, 127).unwrap();
		assert_eq!(frame, [255, 128, 0, 0, 0, 0, 0, 255]);
	}

	#[test]
	fn master_fader_scales_all_dmx_channels() {
		let mut state = ControlState::default();
		state.apply_fader(0, 127);
		state.apply_fader(7, 127);

		let frame = state.apply_fader(8, 64).unwrap();
		assert_eq!(frame, [128, 0, 0, 0, 0, 0, 0, 128]);
	}

	#[test]
	fn ignores_fader_indexes_outside_channel_and_master_range() {
		let mut state = ControlState::default();
		assert_eq!(state.apply_fader(9, 127), None);
	}
}
