use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use walkdir::WalkDir;
use std::io::{self, Write};

fn compress_videos(input_dir: &Path, output_dir: &Path) -> io::Result<()> {
    // Create the output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Collect all video files in the input directory
    let video_files: Vec<PathBuf> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    // Abort if no video files are found
    if video_files.is_empty() {
        eprintln!("No video files found in the input directory.");
        return Ok(());
    }

    // Initialize the progress bar
    let pb = ProgressBar::new(video_files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    for video_file in video_files {
        let output_file = output_dir.join(video_file.file_name().unwrap());
        let log_file = output_file.with_extension("log");

        // Compress the video using ffmpeg with hevc_nvenc
        let ffmpeg_status = ProcessCommand::new("ffmpeg")
            .args([
                "-i",
                video_file.to_str().unwrap(),
                "-movflags",
                "use_metadata_tags",
                "-map_metadata",
                "0",
                "-vcodec",
                "hevc_nvenc",
                "-preset",
                "fast",
                "-crf",
                "28",
                "-c:a",
                "copy",
                output_file.to_str().unwrap(),
            ])
            .stdout(fs::File::create(&log_file)?) // Redirect stdout to log file
            .stderr(fs::File::create(&log_file)?) // Redirect stderr to log file
            .status()?;

        if ffmpeg_status.success() {
            pb.inc(1); // Update the progress bar
        } else {
            eprintln!("Failed to process file: {:?}", video_file);
        }
    }

    pb.finish_with_message("Video compression completed.");
    Ok(())
}

fn delete_original_files(video_files: Vec<PathBuf>) -> io::Result<()> {
    // If there's no video_files to delete, return early.
    if video_files.is_empty() {
        return Ok(());
    }

    print!("Do you want to delete the original files? (Y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("Y") {
        for video_file in video_files {
            fs::remove_file(&video_file)?;
            println!("Deleted file: {:?}", video_file);
        }
        println!("Original files deleted.");
    }

    Ok(())
}

fn main() -> io::Result<()> {
    // Define and parse command-line arguments using clap
    let matches = Command::new("Video Compressor")
        .version("1.0")
        .author("Eduardo Rezaghi <eduardo.rezaghi@gmail.com>")
        .about("Compresses video files in a directory using ffmpeg with hevc_nvenc")
        .arg(
            Arg::new("input_directory")
                .short('i')
                .long("input")
                .value_name("INPUT_DIRECTORY")
                .help("Specifies the input directory containing video files")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("output_directory")
                .short('o')
                .long("output")
                .value_name("OUTPUT_DIRECTORY")
                .help("Specifies the output directory for the compressed videos")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();

    // Extract input and output directories from the arguments
    let input_directory = Path::new(matches.get_one::<PathBuf>("input_directory").unwrap());
    let output_directory = Path::new(matches.get_one::<PathBuf>("output_directory").unwrap());

    compress_videos(input_directory, output_directory)?;

    // Optionally delete original files
    let video_files: Vec<PathBuf> = WalkDir::new(input_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    delete_original_files(video_files)?;

    Ok(())
}
