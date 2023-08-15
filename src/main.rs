mod audio;
use gtts::save_to_file;
use reqwest;
use rodio::*;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::env;
use url::Url;

use audio::audio::{get_output_stream,play};


#[derive(Debug, Deserialize)]
struct BeamGroup {
   id: String,
   space_id: String,
   name: String,
   beam_instance_ids: Vec<String>,
}


async fn fetch_beam_groups(_base_url: &Url) -> Result<Vec<BeamGroup>, Box<dyn Error>> {
   let endpoint = _base_url.join("beam-groups")?;
   let response: Vec<BeamGroup> = reqwest::get(endpoint).await?.json().await?;
   Ok(response)
}


#[tokio::main]
async fn main() {
   let base_url_str = env::var("NEBULA_API_URL").unwrap_or_else(|_| "http://localhost:5555".to_string());
   let base_url = Url::parse(&base_url_str).expect("Failed to parse BASE_URL");

   let current_dir = env::current_dir().unwrap();
   println!("Current directory: {}", current_dir.display());

   let mut has_beam_groups = true;
   match fetch_beam_groups(&base_url).await {
      Ok(beam_groups) => {
         for group in beam_groups {
            println!("{:?}", group);
         }
      }
      Err(e) => {
         println!("Error: {}", e);
         has_beam_groups = false;
      }
   }

   // If has beam groups, use them to figure out which channels to play.
   println!("Found system beam groups: {}", has_beam_groups);

   // list_host_devices();
   let (_stream, stream_handle) = get_output_stream("default");

   let sink = Sink::try_new(&stream_handle).unwrap();

   // For each beam or audio routing, generate a tts file and then play it. 
   let channels = 2;
   let zeros = vec![0.0f32; channels];
   for i in 0..channels {
      let mut ch_gains = zeros.clone();
      ch_gains[i] = 1.0;
      let ch = i + 1;
      save_to_file(ch.to_string().as_str(), "/home/jjy/Workspace/soundcheck/resources/to_play.mp3");
      let file = File::open("/home/jjy/Workspace/soundcheck/resources/sound.mp3").unwrap();
      let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
      let source_buffer = source.buffered();
      play(&sink, &source_buffer, ch_gains);
   
   }
   sink.sleep_until_end();
}