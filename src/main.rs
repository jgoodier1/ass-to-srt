use std::env;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
struct ASSDialog {
    start: String,
    end: String,
    text: String,
}

fn get_ass_files(dir: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut ass_vec = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_path = entry.path();
        if !file_path.is_dir() {
            let extension = file_path.extension();
            match extension {
                Some(extension) => {
                    if extension == "ass" {
                        ass_vec.push(file_path);
                    }
                }
                None => {
                    continue;
                }
            }
        }
    }
    Ok(ass_vec)
}

fn get_ass_lines(lines: std::io::Lines<std::io::BufReader<std::fs::File>>) -> Vec<ASSDialog> {
    let mut line_vector = Vec::new();

    for line in lines {
        if let Ok(line) = line {
            if let Some(dialog_location) = line.find("Dialogue") {
                if dialog_location == 0 {
                    let v: Vec<&str> = line.split(",").collect();

                    let mut start_ms: Vec<&str> = v[1].split(".").collect();
                    let start_ms_as_u16 = start_ms[1]
                        .parse::<u16>()
                        .expect("could not parse starting ms");
                    let milliseconds = &(start_ms_as_u16 * 10).to_string()[..];
                    start_ms[1] = milliseconds;

                    let mut end_ms: Vec<&str> = v[2].split(".").collect();
                    let end_ms_as_u16 = end_ms[1]
                        .parse::<u16>()
                        .expect("could not parse starting ms");
                    let milliseconds = &(end_ms_as_u16 * 10).to_string()[..];
                    end_ms[1] = milliseconds;

                    let dialog = ASSDialog {
                        start: start_ms.join(","),
                        end: end_ms.join(","),
                        // there may be commas in the dialog
                        text: v[9..].join(" ").to_string(),
                    };
                    line_vector.push(dialog);
                }
            };
        };
    }

    line_vector
}

fn main() {
    let dir = env::current_dir().expect("couldn't read current directory");

    let ass_files = match get_ass_files(&dir) {
        Ok(ass_files) => ass_files,
        Err(error) => panic!("couldn't get ass files: {}", error),
    };

    for file in ass_files.into_iter() {
        let open_file = match File::open(&file) {
            Ok(open_file) => open_file,
            Err(error) => panic!("couldn't open the file {:?} because {}", file, error),
        };
        let lines = io::BufReader::new(open_file).lines();

        let ass_lines = get_ass_lines(lines);

        let mut srt_file_name = file
            .file_stem()
            .expect("couldn't get filename")
            .to_str()
            .expect("file stem isn't valid unicode")
            .to_string();
        srt_file_name.push_str(".srt");

        let srt_file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&srt_file_name)
        {
            Ok(srt_file) => srt_file,
            Err(error) => {
                eprintln!("couldn't create file: {}", error);
                continue;
            }
        };

        let mut line_number = 1;
        for dialog in ass_lines.into_iter() {
            writeln!(&srt_file, "{}", &line_number).expect("error writing line number");
            writeln!(&srt_file, "0{} --> 0{}", dialog.start, dialog.end)
                .expect("error writing time");
            writeln!(&srt_file, "{}", dialog.text).expect("error writing dialog");
            writeln!(&srt_file, "").expect("error writing blank line");
            line_number += 1;
        }

        let ass_file_name = file.file_name().expect("error getting ASS file name");

        println!("{:?} -> {:?}", ass_file_name, srt_file_name)
    }
}
