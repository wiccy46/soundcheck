use reqwest;
use std::fs::File;
use std::io::prelude::*;

/// Use google translate to get an audio version of your text and save this in a file.
/// Return true if everything succeeds.
/// Modified from gtts crate, due to minreq not working properly.
pub async fn save_to_file(text: &str, filename: &str) -> bool {
    let len = text.len();

    if let Ok(response) = reqwest::get(&format!("https://translate.google.fr/translate_tts?ie=UTF-8&q={}&tl=en&total=1&idx=0&textlen={}&tl=en&client=tw-ob", text.to_string(), len)).await {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes().await {
                if let Ok(mut file) = File::create(filename) {
                    if file.write_all(&bytes).is_ok() {
                        return true;
                    }
                }
            }
        }
    }

    false
}
