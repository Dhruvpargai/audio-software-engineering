use crate::ring_buffer::RingBuffer;
use crate::lfo::Lfo;

pub struct Vibrato {
    frequency: f32, // Frequency in Hz
    amplitude: f32, // Depth of the LFO - 0  to 100
    sample_rate_hz: f32, // Sample rate in Hz
    lfo: Lfo,
    num_channels: usize,
    delay_lines : Vec<RingBuffer::<f32>>,
    delay: f32 // Delay in samples
    width: f32 // Width of the vibrato in samples
}

impl Vibrato {
    pub fn new(frequency: f32, amplitude: f32, sample_rate_hz: f32, num_channels: usize, delay: f32, width: f32) -> Self {
        let lfo = Lfo::new(frequency, amplitude, sample_rate_hz);
        let delay_in_samples = delay * sample_rate_hz;
        let width_in_samples = width * sample_rate_hz;
        Vibrato {
            frequency,
            amplitude,
            sample_rate_hz,
            lfo,
            num_channels,
            delay_lines: vec![RingBuffer::<f32>::new((delay * sample_rate_hz) as usize); num_channels],
            delay_in_samples,
            width_in_samples
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]], block_size: usize) {
        let lfo_block = self.lfo.get_samples(block_size);
        for (sample_index, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
            for (channel, (input_sample, output_sample)) in input_channel.iter().zip(output_channel.iter_mut()).enumerate() {
                println!("channel {}, sample {} sample_index {}", channel, *input_sample, sample_index);
                
                self.delay_lines[channel].set_read_index(self.delay_in_samples[channel].get_write_index() as i32 - self.delay as i32); // Make safe for negative index
                self.delay_lines[channel].push(*input_sample);

                // Set output_sample fractional delay value. Can't pop it so need to peek
                *output_sample = self.delay_lines[channel].get_frac(lfo_block[sample_index]);

                // Add input value to delayline
                self.delay_lines[channel].push(*input_sample);

                // match self.filter_type {
                //     FIR => {
                //         *output_sample = *input_sample + self.gain * self.delay_lines[channel].pop();
                //         self.delay_lines[channel].push(*input_sample);
                //     }
                //     IIR => {
                //         *output_sample = *input_sample + self.gain * self.delay_lines[channel].pop();
                //         self.delay_lines[channel].push(*output_sample);
                //     }
                // }
            }
        }
    }

    pub fn reset(&mut self) {
        self.delay_lines = vec![RingBuffer::<f32>::new((self.delay) as usize); self.num_channels];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let fs: f32 = 8.0;
        let delay: f32 = 0.25;
        let width: f32 = 0.125;
        let gain: f32 = 1.0; 
        let channels = 2;
        let block_size = 4;
        let fhz: f32 = 2.0;
        let amplitude: f32 = 1.0;
        let mut input_signal = vec![-0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0]; //FHz = 2

        let mut vibrato = Vibrato::new(fhz, amplitude, fs, channels, delay, width);
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
                        for (innerIndex, sample) in channel.iter().enumerate() {
                            // assert_eq!(*sample, 0.0);
                            // println!("{}: {}", innerIndex, sample);
                        }
                    }
                }

                input.clear();
            }
        }
    }
}