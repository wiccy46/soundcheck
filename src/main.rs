extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::thread::sleep;
use std::time::Duration;

fn next_phase_value(phase: &mut f32, sample_rate: f32) -> f32 {
    let frequency = 200.0;
    let amplitude = 0.2;
    *phase = (*phase + 1.0) % sample_rate;
    (*phase * frequency * 2.0 * PI / sample_rate).sin() * amplitude
}

fn make_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: f32,
    channel_to_play_on: usize,
    master_volume: f32,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let mut phase = 0f32;
    let mut next_value = move || next_phase_value(&mut phase, sample_rate);

    let err_fn = |err| eprintln!("An error occurred on the output stream: {}", err);
    device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = next_value();
                for (i, sample) in frame.iter_mut().enumerate() {
                    if i == channel_to_play_on {
                        *sample = master_volume * value;
                    } else {
                        *sample = 0.0;
                    }
                }
            }
        },
        err_fn,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Play a tone on each channel of the output device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Failed to get default output device");
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;
    let master_volume = 0.5;

    println!("Device: {:?}", device.name());
    println!("Channels: {:?}", channels);

    println!("Playing...");
    for channel_to_play in 0..channels {
        let stream = make_stream(
            &device,
            &config.clone().into(),
            channels,
            sample_rate,
            channel_to_play,
            master_volume,
        )?;
        stream.play()?;

        sleep(Duration::from_secs(2));

        stream.pause()?;
        sleep(Duration::from_millis(200));
    }

    println!("Finished...");
    Ok(())
}
