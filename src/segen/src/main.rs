extern crate synthrs;

use synthrs::synthesizer::{ make_samples, quantize_samples };
use synthrs::filter::*;
use synthrs::wave::{sawtooth_wave, square_wave};
use synthrs::writer::write_wav_file;

fn main() {
	macro_rules! write_sound {
		($name: expr, $sample: expr) => {
			write_wav_file(concat!("src/tui/src/se/", $name, ".wav"), 44_100,
				&quantize_samples::<i16>($sample)
			).expect("failed to write to file");
		}
	}

	let sample = &make_samples(0.1, 44_100, |t|
		2.0 * (t * (100. * (80. * t).sin()) * 2.0 * 3.14159).sin()
	);
	let pass = bandpass_filter(
		cutoff_from_frequency(1000.0, 44_100),
		cutoff_from_frequency(3000.0, 44_100),
		0.01
	);
	write_sound!("clear_drop", &convolve(&pass, &sample));

	write_sound!("rotate_fail", &make_samples(0.1, 44_100, |t|
		0.5 * ((
			0.6 * sawtooth_wave(100.)(t) +
			sawtooth_wave(70.)(t)
		) * (1. - t * 10.))
	));
	write_sound!("rotate_twist", &make_samples(0.05, 44_100, |t|
		0.15 * (400. * ((70. * t).sin())).sin()
	));

	let sample = &make_samples(0.1, 44_100, |t|
		0.5 * square_wave(50.)(t) * (1. - t * 10.)
	);
	let pass = lowpass_filter(
		cutoff_from_frequency(400.0, 44_100),
		0.01
	);
	write_sound!("plain_drop", &convolve(&pass, &sample));

	let sample = make_samples(0.05, 44_100, |t|
		0.9 * square_wave(10550.)(t) * (
			if t > 0.01 {
				1. - (t - 0.01) * 20.
			} else {
				100. * t
			}
		)
	);
	let pass = bandpass_filter(
		cutoff_from_frequency(5000.0, 44_100),
		cutoff_from_frequency(8000.0, 44_100),
		0.01
	);
	write_sound!("rotate_regular", &convolve(&pass, &sample));

	write_sound!("attack_drop", &make_samples(0.1, 44_100, |t|
		if t > 0.05 {
			0.3 * sawtooth_wave(3600.)(t) * (1. - t * 10.)
		} else {
			0.3 * sawtooth_wave(3200.)(t) * (1. - t * 10.)
		}
	));
}
