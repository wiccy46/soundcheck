use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::source::UniformSourceIterator;
use rodio::source::{Buffered, ChannelVolume};
use rodio::cpal::SampleRate;
use rodio::*;
use std::fs::File;
use std::io::BufReader;

pub type ResampleBuffer = Buffered<UniformSourceIterator<Decoder<BufReader<File>>, i16>>;

pub fn list_host_devices() {
    println!("---- Listing host devices: ----");
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        println!(" # Device : {}", dev_name);
    }
    println!("---- Completed ----");
}

pub fn get_output_stream(device_name: &str, sr: u32) -> (OutputStream, OutputStreamHandle, u16) {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            println!("Accquiring config with the max channels.");
            let supported_configs = dev.supported_output_configs().unwrap();
            let mut default_config = dev.default_output_config().unwrap();
            for supported_config in supported_configs {
                if supported_config.max_sample_rate() < SampleRate(sr) {
                    continue;
                } else {
                    if supported_config.channels() > default_config.channels() {
                        default_config = supported_config.with_sample_rate(SampleRate(sr));
                    }
                }
            }
            let max_ch = default_config.channels() as u16;

            let (_stream, stream_handle) = OutputStream::try_from_device_config(&dev, default_config).unwrap();

            return (_stream, stream_handle, max_ch);
        }
    }
    panic!("Device not found");
}

pub fn play(sink: &Sink, source: &ResampleBuffer, ch_vol: Vec<f32>) {
    let ch_vol = ChannelVolume::new(source.clone(), ch_vol);
    sink.append(ch_vol);
}
