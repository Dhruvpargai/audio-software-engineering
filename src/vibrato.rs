use crate::ring_buffer::RingBuffer;
use crate::lfo::Lfo;

pub struct Vibrato {
    /// Frequency in Hz
    frequency: f32,
    /// Depth of the LFO - 0.0 to 1.0
    amplitude: f32,
    /// Delay in seconds
    delay: usize,
    /// Sample rate in Hz
    sample_rate_hz: f32,
    /// Number of channels
    num_channels: usize, 
    lfo: Lfo,
    delay_lines : Vec<RingBuffer<f32>>,
}

impl Vibrato {
    /// Initialize a new Vibrato with a given frequency(Hz), amplitude(0.0-1.0), delay(s), sample rate(Hz), and number of channels
    pub fn new(frequency: f32, amplitude: f32, delay: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let lfo = Lfo::new(frequency, amplitude, sample_rate_hz);
        let delay_in_samples = (delay * sample_rate_hz) as usize;
        let amplitude = amplitude.min(1.0); // Ensure amplitude is between 0 and 1
        Vibrato {
            frequency,
            amplitude,
            delay: delay_in_samples,
            sample_rate_hz,
            num_channels,
            lfo,
            delay_lines: vec![RingBuffer::<f32>::new((2 * delay_in_samples) * sample_rate_hz as usize); num_channels],
        }
    }

    /// Process the input and write to the output buffer
    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let mut max_block_size = 0;
        for channel_index in 0..input.len() {
            max_block_size = max_block_size.max(input[channel_index].len());
        }
        let lfo_block = self.lfo.get_samples(max_block_size);
        
        for channel in 0..self.delay_lines.len() {
            for (sample_index, (&input_sample, output_sample)) in input[channel].iter().zip(output[channel].iter_mut()).enumerate() {
                if self.delay_lines[channel].len() > (2 * self.delay - 1) { 
                    let offset_index = lfo_block[sample_index] * (2 * self.delay ) as f32;
                    *output_sample = self.delay_lines[channel].get_frac(offset_index);
                    self.delay_lines[channel].pop();
                } else {
                    *output_sample = input_sample;
                }
                self.delay_lines[channel].push(input_sample);
            }
        }
    }

    /// Reset the Vibrato to its initial state
    pub fn reset(&mut self) {
        self.delay_lines = vec![RingBuffer::<f32>::new((2 * self.delay) * self.sample_rate_hz as usize); self.num_channels];
    }

    /// Set the frequency(Hz) of the Vibrato
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.lfo.set_frequency(frequency);
    }

    /// Set the amplitude of the Vibrato
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude.min(1.0);
        self.lfo.set_amplitude(amplitude);
    }

    /// Set the delay(seconds) of the Vibrato
    pub fn set_delay(&mut self, delay: f32) {
        self.delay = (delay * self.sample_rate_hz) as usize;
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPSILON: f32 = 1e-5;

    fn generate_sine_wave(freq: f32, amp: f32, sample_rate: f32, num_samples: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            samples.push(amp * (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin());
        }
        samples
    }

    #[test]
    fn test_zero_amplitude_modulation() {
        let fs: f32 = 44100.0;
        let delay: f32 = 0.0001;
        let channels = 1;
        let block_size = 1024;
        let fhz: f32 = 2.0;
        let amplitude: f32 = 0.0;
        let delay_in_samples = (fs * delay) as usize;
        let signal_length = (fs * 1.0) as usize;

        let input = vec![generate_sine_wave(200.0, 1.0, fs, signal_length as usize); channels];
        let mut output = vec![vec![0.0; signal_length as usize]; channels];
        let ins: &[&[f32]] = &[&input[0]];
        let outs: &mut[&mut [f32]] = &mut [&mut output[0]];

        let mut vibrato = Vibrato::new(fhz, amplitude, delay, fs, channels);
        vibrato.process(ins, outs);

        for channel in 0..channels {
            for i in 2 * delay_in_samples..signal_length { // Wait for vibrato to kick in
                assert!(f32::abs(input[channel][i - delay_in_samples] - output[channel][i]) <= EPSILON);
            }
        }
    }

    #[test]
    fn test_dc_signal() {
        let fs: f32 = 44100.0;
        let delay: f32 = 0.01;
        let channels = 1;
        let fhz: f32 = 2.0;
        let amplitude: f32 = 1.0;
        let delay_in_samples = (fs * delay) as usize;
        let signal_length = (fs * 1.0) as usize;

        let input = vec![vec![0.5; signal_length as usize]; channels];
        let mut output = vec![vec![0.0; signal_length as usize]; channels];
        let ins: &[&[f32]] = &[&input[0]];
        let outs: &mut[&mut [f32]] = &mut [&mut output[0]];

        let mut vibrato = Vibrato::new(fhz, amplitude, delay, fs, channels);
        vibrato.process(ins, outs);

        for channel in 0..channels {
            for i in 2 * delay_in_samples..signal_length { // Wait for vibrato to kick in
                assert!(f32::abs(input[channel][i] - output[channel][i]) <= EPSILON); // This test fails
            }
        }
    }

    #[test]
    fn test_zero_input() {
        let fs: f32 = 44100.0;
        let delay: f32 = 0.01;
        let channels = 1;
        let fhz: f32 = 2.0;
        let amplitude: f32 = 1.0;
        let delay_in_samples = (fs * delay) as usize;
        let signal_length = (fs * 1.0) as usize;

        let input = vec![vec![0.0; signal_length as usize]; channels];
        let mut output = vec![vec![0.0; signal_length as usize]; channels];
        let ins: &[&[f32]] = &[&input[0]];
        let outs: &mut[&mut [f32]] = &mut [&mut output[0]];

        let mut vibrato = Vibrato::new(fhz, amplitude, delay, fs, channels);
        vibrato.process(ins, outs);

        for channel in 0..channels {
            for i in 2 * delay_in_samples..signal_length { // Wait for vibrato to kick in
                assert!(f32::abs(input[channel][i] - output[channel][i]) <= EPSILON);
            }
        }
    }
}