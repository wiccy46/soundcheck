mod audio;
mod nebula;

use clap::{App, Arg};
use gtts::save_to_file;
use rodio::*;
use std::fs::File;
use std::collections::BTreeMap;
use std::io::BufReader;
use std::{thread, time};
use std::path::Path;
use std::env;
use url::Url;

use rodio::source::UniformSourceIterator;

use audio::audio::{get_output_stream, list_host_devices, play, ResampleBuffer};
use nebula::nebula::active_channels;

fn cleanup(f: &Path) {
    if Path::new(&f).exists() {
        std::fs::remove_file(&f).unwrap();
    } else {
        println!("File {} does not exist", f.display());
    }
}

fn gain_validator(val: String) -> Result<(), String> {
    match val.parse::<f32>() {
        Ok(v) if v >= 0.0 && v <= 1.0 => Ok(()),
        _ => Err(String::from("Gain must be a float between 0.0 and 1.0")),
    }
}


fn play_loop(channels: usize, channel_map: &BTreeMap<u16, String>, gain: f32, device_sr: u32, filepath: &Path, sink: &Sink, receiver_mode: bool) {
    let zeros = vec![0.0f32; channels];
    for (key, value) in channel_map.iter() {
        let i = key.clone() as usize;
        if i <= channels {
            let mut ch_gains = zeros.clone();
            ch_gains[i] = gain;
            let content: String;
            if receiver_mode {
                content = format!("{}, checked", value);
            } else {
                let ch = i + 1;
                content = ch.to_string();
            }

            save_to_file(content.as_str(), filepath.to_str().unwrap());
            let file = File::open(filepath.clone()).unwrap();
            let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

            let resample: UniformSourceIterator<Decoder<BufReader<File>>, i16> =
                UniformSourceIterator::new(source, 1, device_sr);
            let resample_buffer: ResampleBuffer = resample.buffered();

            play(&sink, &resample_buffer, ch_gains);
            thread::sleep(time::Duration::from_millis(1000));
        }
    }
}


#[tokio::main]
async fn main() {
    let app = App::new("Soundcheck")
        .version("0.1.0")
        .author("Jiajun Yang")
        .about("Play sound on specific channels to make sure there are sound.")
        .arg(
            Arg::with_name("gain")
                .short("g")
                .long("gain")
                .value_name("GAIN")
                .help("Sets the linear gain, range 0 -- 1.0")
                .takes_value(true)
                .default_value("1.0")
                .validator(gain_validator),
        )
        .arg(
            Arg::with_name("device")
                .short("d")
                .long("device")
                .value_name("DEVICE")
                .help("Device name to play on, use --help to list available devices")
                .takes_value(true)
                .default_value("default"),
        )
        .arg(
            Arg::with_name("receivers")
                .short("r")
                .long("receivers")
                .value_name("RECEIVERS")
                .help("If set to true, speake out the name of the receivers")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("help")
                .short("h")
                .long("help")
                .help("Display help information and list available devices"),
        );
    let matches = app.clone().get_matches();

    if matches.is_present("help") {
        let mut buffer = Vec::new();
        app.write_help(&mut buffer).unwrap();
        println!("{}", String::from_utf8(buffer).unwrap());
        list_host_devices();
        return;
    }
    let gain: f32 = matches
        .value_of("gain")
        .unwrap()
        .parse()
        .expect("Expect float number");

    // make sure url ends with a slash
    let base_url_str =
        env::var("NEBULA_API_URL").unwrap_or_else(|_| "http://localhost:5555".to_string());
    let base_url = Url::parse(&base_url_str).expect("Failed to parse BASE_URL");

    let device = matches.value_of("device").unwrap();
    let current_dir = env::current_dir().unwrap();
    let filepath = current_dir.join("resources/to_play.mp3");

    let mut active_channels_map: BTreeMap<u16, String> = BTreeMap::new();

    let (_stream, stream_handle, device_sr, default_outputs) = get_output_stream(&device);
    println!("Device sample rate: {}", device_sr);
    println!("Device default outputs: {}", default_outputs);

    // Look at the Nebula API to find active channels 
    let ac_result = active_channels(&base_url).await; 

    match ac_result {
        Ok(ac) => {
            active_channels_map = ac;
            println!("Active channels: ");
            for (k, v) in &active_channels_map {
                println!("{}: {}", v, k + 1);
            }
        }
        Err(e) => {
            // Create a default channel map based on default outputs
            for i in 0..default_outputs {
                active_channels_map.insert(i, i.to_string());
            }
            println!("Failed to find active channels, play on all default channels.");
            println!("Error: {}", e);
        }
    }
        
    let sink = Sink::try_new(&stream_handle).unwrap();


    let receiver_mode = matches.is_present("receivers");

    // For each beam or audio routing, generate a tts file and then play it.
    for _ in 0..1 {
        play_loop(
            default_outputs as usize,
            &active_channels_map,
            gain, 
            device_sr, 
            &filepath, 
            &sink,
            receiver_mode
        );
    }

    sink.sleep_until_end();
    cleanup(&filepath);
}
