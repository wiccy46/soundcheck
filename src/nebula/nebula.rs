use reqwest;
use serde::Deserialize;
use std::error::Error;
use url::Url;


#[derive(Debug, Deserialize)]
pub struct Beam {
    id: String,
    name: String,
}


#[derive(Debug, Deserialize)]
pub struct BeamInstance {
    beam_group_id: String,
    beam_id: String,
    id: String,
}


#[derive(Debug, Deserialize)]
pub struct BeamGroup {
    id: String,
    name: String,
    beam_instance_ids: Vec<String>,
}


#[derive(Debug, Deserialize)]
struct Space {
    active_preset_id: String,
}


#[derive(Debug, Deserialize)]
struct Preset {
    beam_group_ids: Vec<String>,
    name: String,
}


#[derive(Debug, Deserialize)]
struct AudioInput {
    id: String,
    name: String,
    index: u16,
}


async fn fetch_preset_id(_base_url: &Url, id: &String) -> Result<Preset, Box<dyn Error>> {
    let param = "presets/";
    let endpoint = _base_url.join(&format!("{}{}", param, id))?;
    println!("Endpoint: {}", endpoint);
    let response: Preset = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}


async fn fetch_spaces(_base_url: &Url) -> Result<Vec<Space>, Box<dyn Error>> {
    let param = "spaces";
    let endpoint = _base_url.join(param)?;
    let response: Vec<Space> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}


async fn fetch_beam_groups(_base_url: &Url) -> Result<Vec<BeamGroup>, Box<dyn Error>> {
    let param = "beam-groups";
    let endpoint = _base_url.join(param)?;
    let response: Vec<BeamGroup> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}


async fn fetch_beam_instances(_base_url: &Url) -> Result<Vec<BeamInstance>, Box<dyn Error>> {
    let param = "beam-instances";
    let endpoint = _base_url.join(param)?;
    let response: Vec<BeamInstance> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}

async fn fetch_beams(_base_url: &Url) -> Result<Vec<Beam>, Box<dyn Error>> {
    let param = "beams";
    let endpoint = _base_url.join(param)?;
    let response: Vec<Beam> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}


async fn fetch_audio_inputs(_base_url: &Url) -> Result<Vec<AudioInput>, Box<dyn Error>> {
    let param = "audio-inputs";
    let endpoint = _base_url.join(param)?;
    let response: Vec<AudioInput> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}

pub async fn active_beams(_base_url: &Url) -> Result<(), String>{
    let mut preset: String = String::from("");
    let mut beam_groups_ids: Vec<String> = Vec::new();
    match fetch_spaces(&_base_url).await {
        Ok(spaces) => {
            let fst = spaces.first().unwrap();
            // Handle active_preset_id being null
            if fst.active_preset_id.is_empty() {
                println!("Active preset id is empty");
                return Err("active preset is empty".to_string());
            }
            preset = fst.active_preset_id.clone();
            println!("Active preset id: {}", preset);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    match fetch_preset_id(&_base_url, &preset).await {
        Ok(preset) => {
            beam_groups_ids = preset.beam_group_ids;
            println!("Active preset name: {}", preset.name);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let mut beam_instances_ids: Vec<String> = Vec::new();

    match fetch_beam_groups(&_base_url).await {
        Ok(beam_groups) => {
            for bg in beam_groups {
                if beam_groups_ids.contains(&bg.id) {
                    println!("Beam group id: {}", bg.id);
                    println!("Beam group name: {}", bg.name);
                    println!("Beam group beam instance ids: {:?}", bg.beam_instance_ids);
                    beam_instances_ids.extend(bg.beam_instance_ids);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let mut beam_ids: Vec<String> = Vec::new();

    match fetch_beam_instances(&_base_url).await {
        Ok(beam_instances) => {
            for bi in beam_instances {
                if beam_instances_ids.contains(&bi.id) {
                    println!("Beam instance beam id: {}", bi.beam_id);
                    beam_ids.push(bi.beam_id);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let mut beam_names: Vec<String> = Vec::new();

    match fetch_beams(&_base_url).await {
        Ok(beams) => {
            for b in beams {
                if beam_ids.contains(&b.id) {
                    println!("Beam name: {}", b.name);
                    beam_names.push(b.name);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    match fetch_audio_inputs(&_base_url).await {
        Ok(audio_inputs) => {
            for ai in audio_inputs {
                if beam_names.contains(&ai.name) {
                    println!("Audio input id: {}", ai.id);
                    println!("Audio input name: {}", ai.name);
                    println!("Audio input index: {}", ai.index);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }



    return Ok(());
}