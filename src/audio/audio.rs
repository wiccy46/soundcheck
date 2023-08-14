use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::source::Buffered;
use rodio::source::ChannelVolume;
use rodio::*;
use std::fs::File;
use std::io::BufReader;

pub fn list_host_devices() {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        println!(" # Device : {}", dev_name);
    }
}

pub fn get_output_stream(device_name: &str) -> (OutputStream, OutputStreamHandle) {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    let (mut _stream, mut stream_handle) = OutputStream::try_default().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            (_stream, stream_handle) = OutputStream::try_from_device(&dev).unwrap();
        }
    }
    return (_stream, stream_handle);
}

pub fn play(sink: &Sink, source: &Buffered<Decoder<BufReader<File>>>, ch_vol: Vec<f32>) {
    let ch_vol = ChannelVolume::new(source.clone(), ch_vol);
    sink.append(ch_vol);
}
