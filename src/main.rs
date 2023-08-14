mod audio;
use std::fs::File;
use std::io::BufReader;
use rodio::*;

use audio::audio::{get_output_stream,play};

fn main() {
   // list_host_devices();
   let (_stream, stream_handle) = get_output_stream("default");

   let sink = Sink::try_new(&stream_handle).unwrap();
   let file = File::open("/home/jjy/Workspace/soundcheck/resources/sound.mp3").unwrap();
   let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
   let source_buffer = source.buffered();

   let channel_volumes = vec![0.0, 1.0];
   play(&sink, &source_buffer, channel_volumes);
   
   let channel_volumes = vec![1.0, 0.0];
   play(&sink, &source_buffer, channel_volumes);

   sink.sleep_until_end();
}