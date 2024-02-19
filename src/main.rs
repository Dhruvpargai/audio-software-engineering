use std::{fs::File, io::Write};
use crate::comb_filter::CombFilter;
use crate::comb_filter::FilterType::*;
use crate::comb_filter::FilterParam::*;
mod comb_filter;
mod ring_buffer;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    let block_size = 1024;
    let mut delay = 0.5;
    let mut gain = 0.5;
    let mut filter_type: comb_filter::FilterType = FIR;
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output text filename>", args[0]);
        return
    }

    match args.get(3) {
        Some(value) => {
            filter_type = match value.as_str() {
                "FIR" => comb_filter::FilterType::FIR,
                "IIR" => comb_filter::FilterType::IIR,
                _ => {
                    eprintln!("Invalid filter type");
                    return
                }
            }
        },
        None => println!("No filter type param given. Using default"),
    }
    
    match args.get(4) {
        Some(value) => {
            delay = match value.parse::<f32>() {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("Invalid delay value");
                    return
                }
            }
        },
        None => println!("No delay param given. Using default"),
    }

    match args.get(5) {
        Some(value) => {
            gain = match value.parse::<f32>() {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("Invalid gain value");
                    return
                }
            }
        },
        None => println!("No gain param given. Using default"),
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let mut writer = hound::WavWriter::create(&args[2], spec).unwrap();

    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:
    let mut input = vec![];
    let mut output = vec![0.0; block_size*channels];
    println!("delay: {}", delay);
    let mut comb_filter = CombFilter::new(filter_type, delay, spec.sample_rate as f32, channels);
    comb_filter.set_param(comb_filter::FilterParam::Gain, gain).unwrap();
    comb_filter.set_param(comb_filter::FilterParam::Delay, delay).unwrap();

    for sample in reader.samples::<i16>() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        input.push(sample);
        if input.len() == block_size * channels {
            let input_slice: Vec<_> = input.chunks(channels).collect();
            let mut output_slice: Vec<_> = output.chunks_mut(channels).collect();
            comb_filter.process(&input_slice, &mut output_slice);
            for channel in output_slice {
                for sample in channel {
                    writer.write_sample((*sample * (1 << 15) as f32 - 1.0) as i16).unwrap();
                }
            }
            input.clear();
        }
    }

    // Read audio data and write it to the output text file (one column per channel)
    // let mut out = File::create(&args[2]).expect("Unable to create file");
    // for (i, sample) in reader.samples::<i16>().enumerate() {
    //     let sample = sample.unwrap() as f32 / (1 << 15) as f32;
    //     write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    // }
}
