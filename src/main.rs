mod audio;
mod nebula;

use clap::{App, Arg};
use gtts::save_to_file;
use rodio::*;
use std::fs::File;
use std::collections::HashMap;
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


fn play_loop(channels: usize, gain: f32, device_sr: u32, filepath: &Path, sink: &Sink) {
    let zeros = vec![0.0f32; channels];
    for i in 0..channels {
        let mut ch_gains = zeros.clone();
        ch_gains[i] = gain;
        let ch = i + 1;

        save_to_file(ch.to_string().as_str(), filepath.to_str().unwrap());
        let file = File::open(filepath.clone()).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

        let resample: UniformSourceIterator<Decoder<BufReader<File>>, i16> =
            UniformSourceIterator::new(source, 1, device_sr);
        let resample_buffer: ResampleBuffer = resample.buffered();

        play(&sink, &resample_buffer, ch_gains);
        thread::sleep(time::Duration::from_millis(1000));
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
            Arg::with_name("channels")
                .short("c")
                .long("channels")
                .value_name("CHANNELS")
                .help("Number of channels to play on")
                .takes_value(true)
                .default_value("2"),
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

    let mut active_channels_map: HashMap<String, i16> = HashMap::new();
    
    let ac_result = active_channels(&base_url).await; 

    let mut has_active_channels: bool = true;

    match ac_result {
        Ok(ac) => {
            active_channels_map = ac;
            println!("Active channels: ");
            for (k, v) in &active_channels_map {
                println!("{}: {}", k, v);
            }
        }
        Err(e) => {
            // Here if error, then don't use active_channels_map, just play some 
            // fixed channels. 
            has_active_channels = false;
            println!("Failed to find active channels, play fix channels instead.");
            println!("Error: {}", e);
        }
    }
        
    let (_stream, stream_handle, device_sr) = get_output_stream(&device);
    println!("Device sample rate: {}", device_sr);

    let sink = Sink::try_new(&stream_handle).unwrap();

    // This is a hack. Currently RME only give 8. Need major rework to the audio 
    // stream to make full use of the channels. May need to do it in pure cpal.

    let device_max_channels = 8;
    // For each beam or audio routing, generate a tts file and then play it.
    let channels = 2;
    for _ in 0..1 {
        play_loop(channels, gain, device_sr, &filepath, &sink);
    }

    sink.sleep_until_end();
    cleanup(&filepath);
}
