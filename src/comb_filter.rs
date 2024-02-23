use crate::ring_buffer::RingBuffer;
use FilterType::*;
use FilterParam::*;

pub struct CombFilter {
    // TODO: your code here
    filter_type: FilterType,
    max_delay_secs: f32,
    sample_rate_hz: f32,
    gain: f32,
    delay: f32,
    num_channels: usize,
    delay_lines : Vec<RingBuffer::<f32>>
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let buffer_size = (max_delay_secs * sample_rate_hz as f32) as usize;
        CombFilter {
            filter_type,
            max_delay_secs,
            sample_rate_hz,
            gain: 1.0,
            delay: 0.5,
            num_channels,
            delay_lines: vec![RingBuffer::<f32>::new(buffer_size); num_channels]
        }
    }

    pub fn reset(&mut self) {
        // println!("reset self.delay: {}", self.delay);
        self.delay_lines = vec![RingBuffer::<f32>::new((self.delay * self.sample_rate_hz) as usize); self.num_channels];
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        for (input_channel, output_channel) in input.iter().zip(output.iter_mut()) {
            for (channel, (input_sample, output_sample)) in input_channel.iter().zip(output_channel.iter_mut()).enumerate() {
                match self.filter_type {
                    FIR => {
                        *output_sample = *input_sample + self.gain * self.delay_lines[channel].pop();
                        self.delay_lines[channel].push(*input_sample);
                    }
                    IIR => {
                        *output_sample = *input_sample + self.gain * self.delay_lines[channel].pop();
                        self.delay_lines[channel].push(*output_sample);
                    }
                }
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            Gain => {
                if value < 0.0 || value > 1.0 {
                    return Err(Error::InvalidValue { param, value });
                } else {
                    self.gain = value;
                }
                self.reset();
                Ok(())
            }
            Delay => {
                if value < 0.0 || value > self.max_delay_secs {
                    return Err(Error::InvalidValue { param, value });
                } else {
                    self.delay = value;
                }
                self.reset();
                Ok(())
            }
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            Gain => self.gain,
            Delay => self.delay,
        }
    }

    // TODO: feel free to define other functions for your own use
}

// TODO: feel free to define other types (here or in other modules) for your own use
