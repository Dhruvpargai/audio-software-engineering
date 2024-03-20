use crate::ring_buffer::RingBuffer;

pub struct Lfo {
    frequency: f32, // Frequency in Hz
    amplitude: f32, // Depth of the LFO - 0  to 100
    fs: f32, // Sample rate in Hz
    buffer: RingBuffer<f32>
}

impl Lfo {
    pub fn new(frequency: f32, amplitude: f32, fs: f32) -> Self {
        let mut buffer = RingBuffer::new(fs as usize);
        for i in 0..fs as usize {
            buffer.push((2.0 * std::f32::consts::PI * frequency * i as f32 / fs).sin() * amplitude);
        }
        Lfo {
            frequency,
            amplitude,
            fs,
            buffer
        }
    }

    fn reset(&mut self) {
        self.buffer = RingBuffer::new(self.fs as usize);
        for i in 0..self.fs as usize {
            self.buffer.push((2.0 * std::f32::consts::PI * self.frequency * i as f32 / self.fs).sin() * self.amplitude);
        }
    }

    pub fn get_samples(&mut self, num_samples: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(self.buffer.pop());
        }
        samples
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.reset();
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo() {
        let fs = 44100.0;
        let frequency = 2.0;
        let amplitude = 1.0;
        let mut lfo = Lfo::new(frequency, amplitude, fs);
        let block_size = 4409;
        for block_index in 0..100 {
            let samples = lfo.get_samples(block_size);
            for i in 0..block_size as usize { 
                assert_eq!(samples[i], (2.0 * std::f32::consts::PI * frequency * ((i + block_index * block_size) % fs as usize) as f32 / fs).sin() * amplitude);
            }
        }
    }
}