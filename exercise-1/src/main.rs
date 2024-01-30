use std::{fs::File, io::Write};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    // First argument is input .wav file, second argument is output text file.
    let args: Vec<String> = std::env::args().collect();
    // TODO: your code here

    // Open the input wave file and determine number of channels
    // TODO: your code here; see `hound::WavReader::open`.
    let mut reader = hound::WavReader::open(&args[1]).unwrap();

    // Read audio data and write it to the output text file (one column per channel)
    // TODO: your code here; we suggest using `hound::WavReader::samples`, `File::create`, and `write!`.
    //       Remember to convert the samples to floating point values and respect the number of channels!
    let mut outputFile = File::create(&args[2]).unwrap();
    dbg!(reader.spec().sample_format);
    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / i16::MAX as f32;
        if i % 2 == 0 {
            write!(outputFile, "{} ", sample).unwrap();
        } else {
            write!(outputFile, "{}\n", sample).unwrap();
        }
    }
}
