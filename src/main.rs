mod audio;
mod nebula;
mod utils;

use clap::{App, Arg};
use rodio::*;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::{thread, time};
use url::Url;

use rodio::source::UniformSourceIterator;

use audio::audio::{get_output_stream, list_host_devices, play, ResampleBuffer};
use audio::gtts::save_to_file;
use nebula::nebula::active_channels;
use utils::utils::{linear_gain_validator, remove};

fn get_generic_channel_map(gain_vec_size: u16) -> BTreeMap<u16, String> {
    let mut active_channels_map: BTreeMap<u16, String> = BTreeMap::new();
    for i in 0..gain_vec_size {
        active_channels_map.insert(i, i.to_string());
    }
    active_channels_map
}

async fn get_holoplot_channel_map(base_url: &Url, gain_vec_size: u16) -> BTreeMap<u16, String> {
    let mut active_channels_map: BTreeMap<u16, String> = BTreeMap::new();
    // Look at the Nebula API to find active channels
    let ac_result = active_channels(base_url).await;
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
            for i in 0..gain_vec_size {
                active_channels_map.insert(i, i.to_string());
            }
            println!("Failed to find active channels, play on all default channels.");
            println!("Error: {}", e);
        }
    }
    active_channels_map
}

// Base on the channel map, play sound on each channel with out channels muted
async fn play_on_each_ch(
    channels: usize,
    channel_map: &BTreeMap<u16, String>,
    gain: f32,
    device_sr: u32,
    filepath: &Path,
    sink: &Sink,
    receiver_mode: bool,
) {
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

            save_to_file(content.as_str(), filepath.to_str().unwrap()).await;
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
            Arg::with_name("device")
                .short("d")
                .long("device")
                .value_name("DEVICE")
                .help("Device name to play on, use --help to list available devices")
                .takes_value(true)
                .default_value("default"),
        )
        .arg(
            Arg::with_name("gain")
                .short("g")
                .long("gain")
                .value_name("GAIN")
                .help("Sets the linear gain, range 0 -- 1.0")
                .takes_value(true)
                .default_value("1.0")
                .validator(linear_gain_validator),
        )
        .arg(
            Arg::with_name("receivers")
                .short("r")
                .long("receivers")
                .value_name("RECEIVERS")
                .help("If the flag is set, speake out the name of the receivers")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("samplerate")
                .short("s")
                .long("samplerate")
                .value_name("SAMPLERATE")
                .help("Sets the samplerate of the output device")
                .takes_value(true)
                .default_value("48000")
                .validator(|x| x.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())),
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
    let filepath = env::current_dir().unwrap().join("resources/to_play.mp3");

    let device = matches.value_of("device").unwrap();
    let sr = matches
        .value_of("samplerate")
        .unwrap()
        .parse()
        .expect("Expect possitive integer.");

    let (_stream, stream_handle, device_ch_count) = get_output_stream(&device, sr);
    println!("Device outputs: {}", device_ch_count);

    let receiver_mode = matches.is_present("receivers");
    // make sure url ends with a slash
    let base_url_str = env::var("NEBULA_API_URL").unwrap_or_else(|_| "".to_string());

    let active_channel_map = if base_url_str.is_empty() {
        println!("No NEBULA_API_URL is set, play on default channels.");
        get_generic_channel_map(device_ch_count)
    } else {
        println!("Detecting system's active audio receivers...");
        let base_url = Url::parse(&base_url_str).expect("Failed to parse BASE_URL");
        get_holoplot_channel_map(&base_url, device_ch_count).await
    };

    // Start playing sound
    let sink = Sink::try_new(&stream_handle).unwrap();
    // For each beam or audio routing, generate a tts file and then play it.
    for _ in 0..1 {
        play_on_each_ch(
            device_ch_count as usize,
            &active_channel_map,
            gain,
            sr,
            &filepath,
            &sink,
            receiver_mode,
        ).await;
    }
    sink.sleep_until_end();

    remove(&filepath);
}
