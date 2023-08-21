extern crate rodio;

use rodio::Source;
use std::collections::VecDeque;
use std::time::Duration;

pub struct PinkNoise {
    pub sample_rate: usize,
    pub samples_left: usize,
    pub rows: VecDeque<f32>,
    pub accumulator: f32,
}

impl PinkNoise {
    pub fn new(sr: usize, samples: usize) -> Self {
        PinkNoise {
            sample_rate: sr,
            samples_left: samples,
            rows: VecDeque::from(vec![0.0; 16]),
            accumulator: 0.0,
        }
    }
}

impl Iterator for PinkNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.samples_left == 0 {
            None
        } else {
            let white = rand::random::<f32>() * 2.0 - 1.0;
            self.accumulator += white;
            self.accumulator -= self.rows.pop_back().unwrap_or(0.0);
            self.rows.push_front(self.accumulator);
            self.samples_left -= 1;
            Some(self.accumulator / 16.0) // Divided to keep it within [-1, 1] range
        }
    }
}

impl Source for PinkNoise {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples_left)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs(2))
    }
}
