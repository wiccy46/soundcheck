use std::fs::remove_file;
use std::path::Path;

pub fn remove(f: &Path) {
    if Path::new(&f).exists() {
        remove_file(&f).unwrap();
    } else {
        println!("File {} does not exist", f.display());
    }
}

pub fn linear_gain_validator(val: String) -> Result<(), String> {
    match val.parse::<f32>() {
        Ok(v) if v >= 0.0 && v <= 1.0 => Ok(()),
        _ => Err(String::from("Gain must be a float between 0.0 and 1.0")),
    }
}
