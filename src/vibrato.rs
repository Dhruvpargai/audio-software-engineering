use crate::ring_buffer::RingBuffer;
use crate::lfo::Lfo;

pub struct Vibrato {
    frequency: f32, // Frequency in Hz
    amplitude: f32, // Depth of the LFO - 0.0 to 1.0
    delay: usize, // Delay in samples
    sample_rate_hz: f32, // Sample rate in Hz
    block_size: usize,
    num_channels: usize, 
    lfo: Lfo,
    delay_lines : Vec<RingBuffer::<f32>>,
}

impl Vibrato {
    pub fn new(frequency: f32, amplitude: f32, delay: f32, sample_rate_hz: f32, block_size: usize, num_channels: usize) -> Self {
        let lfo = Lfo::new(frequency, amplitude, sample_rate_hz, block_size);
        let delay_in_samples = (delay * sample_rate_hz) as usize;
        let amplitude = amplitude.min(1.0); // Ensure amplitude is between 0 and 1
        Vibrato {
            frequency,
            amplitude,
            delay: delay_in_samples,
            sample_rate_hz,
            block_size,
            num_channels,
            lfo,
            delay_lines: vec![RingBuffer::<f32>::new((2 * delay_in_samples + 1) * sample_rate_hz as usize); num_channels],
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let lfo_block = self.lfo.get_samples();
        for channel in 0..self.delay_lines.len() {
            for (sample_index, (&input_sample, output_sample)) in input[channel].iter().zip(output[channel].iter_mut()).enumerate() {
                if self.delay_lines[channel].len() > (2 * self.delay + 1) { 
                    let offset_index = lfo_block[sample_index] * (2 * self.delay) as f32;
                    *output_sample = self.delay_lines[channel].get_frac(offset_index);
                    self.delay_lines[channel].pop();
                } else {
                    *output_sample = input_sample;
                }
                self.delay_lines[channel].push(input_sample);
            }
        }
    }

    pub fn reset(&mut self) {
        self.delay_lines = vec![RingBuffer::<f32>::new((2 * self.delay + 1) * self.sample_rate_hz as usize); self.num_channels];
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.lfo.set_frequency(frequency);
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude.min(1.0);
        self.lfo.set_amplitude(amplitude);
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay = (delay * self.sample_rate_hz) as usize;
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let fs: f32 = 8.0;
        let delay: f32 = 0.5;
        let channels = 1;
        let block_size = 4;
        let fhz: f32 = 2.0;
        let amplitude: f32 = 1.0;
        let input_signal = vec![-0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0]; //FHz = 2

        let mut vibrato = Vibrato::new(fhz, amplitude, delay, fs, block_size, channels);
        let mut input = vec![0.0; block_size*channels];
        let mut output = vec![0.0; block_size*channels];
        input.clear();

        for (index, sample) in input_signal.iter().enumerate() {
            input.push(*sample);
            if input.len() == block_size * channels {
                let input_slice: Vec<_> = input.chunks(channels).collect();
                let mut output_slice: Vec<_> = output.chunks_mut(channels).collect();
                vibrato.process(&input_slice, &mut output_slice, block_size);

                if index > (fs / fhz) as usize { // We want to do this because we want to wait for the feedback to kick in 
                    for channel in output_slice {
                        for (inner_index, sample) in channel.iter().enumerate() {
                            // assert_eq!(*sample, 0.0);
                            println!("{} {}: {}", index, inner_index, sample);
                        }
                    }
                }

                input.clear();
            }
        }
    }
}