/* 
    TODO:
    BPM frames for songs with different rhythms?
    If there's been a transient in the last n samples, it shouldn't be able to detect more
    Figure out how to read the analysis file into the variables
*/
/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype
// hound is a wav file reading library
extern crate hound;
// file operations
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
// use std::io::BufReader;
// use std::io::BufRead;
use std::collections::VecDeque;

// note: static variables are thread-local
// global variables for tweaking how the detection works
static AVG_LEN: usize = 256;
static MIN_TRANSIENT_GAP: usize = 256;
struct SoundFile {
    /// Sound samples
    samples: Vec<f32>,
    //the name of the file that was read into SoundFile
    file_name: String,
    fs: usize,
    power_buf: VecDeque<f32>,
    analysis: Analysis,
    transient_gap: usize,
}
#[allow(dead_code)]
impl SoundFile {
    fn remove_file_extension(&mut self) {
        // splits the string in 2 at the . sign
        let split: Vec<&str> = self.file_name.splitn(2, '.').collect();
        self.file_name = split[0].to_string();
    }

    fn load_sound(&mut self, path: String) {
        self.file_name = path;
        let mut reader = hound::WavReader::open(self.file_name.clone()).unwrap();
        self.samples = reader.samples().collect::<Result<Vec<_>, _>>().unwrap();
        self.remove_file_extension();
    }
    // GET THIS TO WORK
    fn search_for_file(&self) -> bool {
        // name should be file_name with .txt instead of .wav
        let name = format!("{}.txt", self.file_name);
        Path::new(&name).exists()
    }
    fn generate_analysis_file(&mut self) {
        println!("{}", self.file_name);
        let name = format!("{}.txt", self.file_name);
        //FIXME: Only works if the file exists
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(name)
            .expect("Filen kunne ikke åbnes");
        // file should be filled with the attributes in the AnalysisFile created
        let string: String = format!("{}\n{:?}", self.analysis.tempo, self.analysis.rhythm);
        file.write(string.as_bytes())
            .expect("Der kunne ikke skrives til filen");
    }
    fn bpm_from_rhythm(&mut self) {
        let mut transientsum = 0;
        for i in 0..self.analysis.rhythm.len() {
            if self.analysis.rhythm[i] != 0. {
                transientsum += 1;
            }
        }
        println!("transientsum is {}!", transientsum);
        println!(
            "number of minutes is {}!",
            (self.analysis.rhythm.len() as f32 / self.fs as f32 / 60.)
        );

        // average transients per second * 60 gives us our bpm
        self.analysis.tempo =
            transientsum as f32 / (self.analysis.rhythm.len() as f32 / self.fs as f32 / 60.);
        // limiting bpm to a rational interval 
        while self.analysis.tempo > 200. || self.analysis.tempo < 70. {
            if self.analysis.tempo > 200. {
                self.analysis.tempo /= 2.;
            }
            else {
                self.analysis.tempo *= 2.;
            }
        }
    }
    fn detect_transients(&mut self) {
        self.analysis.rhythm = vec![0.; self.samples.len()];
        let mut power_avg: f32;
        let mut sum: usize = 0;
        for i in 0..self.samples.len() {
            self.power_buf.push_back(self.samples[i]);
            self.power_buf.pop_front();
            // power_avg is actually rms right now. Test to see if it's a good idea
            power_avg = self.power_buf.iter().map(|&x| x.powi(2)).sum::<f32>()
                / self.power_buf.len() as f32;
            // println!("power_avg is {}", power_avg);
            if self.samples[i] - power_avg > 0.7 && self.transient_gap > MIN_TRANSIENT_GAP {
                self.analysis.rhythm[i] = 1.;
                self.transient_gap = 0;
                sum += 1;
            }
            self.transient_gap += 1;
        }
        println!("Sum is {}", sum);
    }
}
impl Default for SoundFile {
    fn default() -> SoundFile {
        let mut vecdeque = VecDeque::new();
        //rms found over 80 samples
        for _i in 0..AVG_LEN { 
            vecdeque.push_back(0.);
        }
        SoundFile {
            samples: vec![0.],
            file_name: format!(""),
            fs: 44100,
            power_buf: vecdeque,
            analysis: Analysis::default(),
            transient_gap: 0,
        }
    }
}
/// contains the information that controls the lightshow. Found on background of SoundFile
struct Analysis {
    /// tempo in beats per minute
    tempo: f32,
    /// Contains every time a transient is detected. Same time format as the SoundFile
    rhythm: Vec<f32>,
}
impl Analysis {
    fn _read_analysis_file(&mut self) {}
}
impl Default for Analysis {
    fn default() -> Analysis {
        Analysis {
            tempo: 0.,
            rhythm: vec![0.],
        }
    }
}

fn main() {
    let mut sound = SoundFile::default();
    // let mut analysis = Analysis { tempo: 2000., rhythm: vec![0.]};
    sound.load_sound(
        r"C:\Users\rasmu\Documents\RustProjects\Projekt4\Tempo\Songs\busybeat100.wav".to_string(),
    );

    if sound.search_for_file() != true {
        sound.detect_transients();
        sound.bpm_from_rhythm();
        sound.generate_analysis_file();
    } else {
        sound.detect_transients();
        sound.bpm_from_rhythm();
        println!("BPM is {}", sound.analysis.tempo);
        sound.generate_analysis_file();
        println!("already exists boy");
    }
    println!("Hello, world!");
}
