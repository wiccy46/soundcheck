use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::source::UniformSourceIterator;
use rodio::source::{Buffered, ChannelVolume, Source};
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

pub fn get_output_stream(device_name: &str) -> (OutputStream, OutputStreamHandle, u32) {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    let (mut _stream, mut stream_handle) = OutputStream::try_default().unwrap();
    let mut sr = 0;
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            sr = dev.default_output_config().unwrap().sample_rate().0;
            (_stream, stream_handle) = OutputStream::try_from_device(&dev).unwrap();
        }
    }
    return (_stream, stream_handle, sr);
}

pub fn play(sink: &Sink, source: &ResampleBuffer, ch_vol: Vec<f32>) {
    let ch_vol = ChannelVolume::new(source.clone(), ch_vol);
    sink.append(ch_vol);
}
