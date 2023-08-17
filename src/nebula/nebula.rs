use reqwest;
use serde::Deserialize;
use std::{collections::HashMap, error::Error};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct BeamInstance {
    beam_id: String,
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct BeamGroup {
    id: String,
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
}

#[derive(Debug, Deserialize)]
struct BeamAudioInput {
    beam_instance_id: String,
    audio_input_id: String,
}

#[derive(Debug, Deserialize)]
struct AudioInputStreamMapping {
    audio_input_id: String,
    channel: u16,
}

async fn fetch_preset_id(_base_url: &Url, id: &String) -> Result<Preset, Box<dyn Error>> {
    let param = "presets/";
    let endpoint = _base_url.join(&format!("{}{}", param, id))?;
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

async fn fetch_audio_inputs(_base_url: &Url) -> Result<Vec<AudioInput>, Box<dyn Error>> {
    let param = "audio-inputs";
    let endpoint = _base_url.join(param)?;
    let response: Vec<AudioInput> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}

async fn fetch_beam_audio_inputs(_base_url: &Url) -> Result<Vec<BeamAudioInput>, Box<dyn Error>> {
    let param = "beam-audio-inputs";
    let endpoint = _base_url.join(param)?;
    let response: Vec<BeamAudioInput> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}

async fn fetch_audio_input_stream_mappings(
    _base_url: &Url,
) -> Result<Vec<AudioInputStreamMapping>, Box<dyn Error>> {
    let param = "audio-input-stream-mappings";
    let endpoint = _base_url.join(param)?;
    let response: Vec<AudioInputStreamMapping> = reqwest::get(endpoint).await?.json().await?;
    Ok(response)
}

pub async fn active_channels(_base_url: &Url) -> Result<HashMap<String, u16>, String> {
    println!("Finding channel routing ...");
    let mut preset: String = String::from("");
    let mut beam_groups_ids: Vec<String>;

    let spaces = fetch_spaces(&_base_url).await.map_err(|e| e.to_string())?;

    let fst = spaces.first().unwrap();
    // Handle active_preset_id being null
    if fst.active_preset_id.is_empty() {
        println!("Active preset id is empty");
        return Err("active preset is empty".to_string());
    }
    preset = fst.active_preset_id.clone();

    let preset_res = fetch_preset_id(&_base_url, &preset)
        .await
        .map_err(|e| e.to_string())?;
    beam_groups_ids = preset_res.beam_group_ids;
    println!("Active preset name: {}", preset_res.name);

    let mut beam_instances_ids: Vec<String> = Vec::new();

    let beam_groups = fetch_beam_groups(&_base_url)
        .await
        .map_err(|e| e.to_string())?;
    for bg in beam_groups {
        if beam_groups_ids.contains(&bg.id) {
            beam_instances_ids.extend(bg.beam_instance_ids);
        }
    }

    let mut beam_ids: Vec<String> = Vec::new();

    let beam_instances = fetch_beam_instances(&_base_url)
        .await
        .map_err(|e| e.to_string())?;
    for bi in beam_instances {
        if beam_instances_ids.contains(&bi.id) {
            beam_ids.push(bi.beam_id);
        }
    }

    let mut audio_input_ids: Vec<String> = Vec::new();

    let beam_audio_inputs = fetch_beam_audio_inputs(&_base_url)
        .await
        .map_err(|e| e.to_string())?;
    for bai in beam_audio_inputs {
        if beam_instances_ids.contains(&bai.beam_instance_id) {
            if !audio_input_ids.contains(&bai.audio_input_id) {
                audio_input_ids.push(bai.audio_input_id);
            }
        }
    }

    let mut active_audio_input_ids: Vec<String> = Vec::new();
    let mut active_audio_input_map: HashMap<String, String> = HashMap::new();

    let audio_inputs = fetch_audio_inputs(&_base_url)
        .await
        .map_err(|e| e.to_string())?;
    for ai in audio_inputs {
        if audio_input_ids.contains(&ai.id) {
            active_audio_input_ids.push(ai.id.clone());
            active_audio_input_map.insert(ai.id, ai.name.clone());
        }
    }

    let mut active_channels: HashMap<String, u16> = HashMap::new();

    let audio_input_stream_mappings = fetch_audio_input_stream_mappings(&_base_url)
        .await
        .map_err(|e| e.to_string())?;
    for aism in audio_input_stream_mappings {
        if active_audio_input_ids.contains(&aism.audio_input_id) {
            active_channels.insert(
                active_audio_input_map
                    .get(&aism.audio_input_id)
                    .unwrap()
                    .clone(),
                aism.channel as u16,
            );
        }
    }

    return Ok(active_channels);
}
