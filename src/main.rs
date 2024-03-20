use std::{fs::File};
use crate::vibrato::Vibrato;

mod ring_buffer;
mod vibrato;
mod lfo;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: {} <input wave filename> <output text filename> <frequency> <amplitude> <delay>", args[0]);
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let sample_rate = spec.sample_rate as f32;

    let block_size = 1024;
    let fhz: f32 = args[3].parse().unwrap();
    let amp: f32 = args[4].parse().unwrap();
    let delay: f32 = args[5].parse().unwrap();
    let mut vibrato = Vibrato::new(fhz, amp, delay, sample_rate, channels);

    let out = File::create(&args[2]).expect("Unable to create file");
    let mut writer = hound::WavWriter::new(out, spec).unwrap();

    let mut block = vec![Vec::<f32>::with_capacity(block_size); channels];
    let mut output_block = vec![vec![0.0_f32; block_size]; channels];
    let num_samples = reader.len() as usize;

    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        block[i % channels].push(sample);
        if (i % (channels * block_size) == 0) || (i == num_samples - 1) {
            let ins = block.iter().map(|c| c.as_slice()).collect::<Vec<&[f32]>>();
            let mut outs = output_block.iter_mut().map(|c| c.as_mut_slice()).collect::<Vec<&mut [f32]>>();
            vibrato.process(ins.as_slice(), outs.as_mut_slice());
            for j in 0..(channels * block[0].len()) {
                writer.write_sample((output_block[j % channels][j / channels] * (1 << 15) as f32) as i32).unwrap();
            }
            for channel in block.iter_mut() {
                channel.clear();
            }
        }
    }
}
