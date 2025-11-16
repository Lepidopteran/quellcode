use std::{
    env::var,
    fs::{read_dir, remove_file},
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{self, Stdio},
    time::Duration,
};

use cargo_metadata::MetadataCommand;
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

const TICKS: &[&str] = &[
    "⠁", "⠂", "⠄", "⡀", "⡈", "⡐", "⡠", "⣀", "⣁", "⣂", "⣄", "⣌", "⣔", "⣤", "⣥", "⣦", "⣮", "⣶", "⣷",
    "⣿", "⡿", "⠿", "⢟", "⠟", "⡛", "⠛", "⠫", "⢋", "⠋", "⠍", "⡉", "⠉", "⠑", "⠡", "⢁",
];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Regenerate typescript bindings
    RegenerateBindings,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::RegenerateBindings => regenerate_bindings(args),
    }
}

fn quellcode_dir() -> PathBuf {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to get metadata");

    metadata
        .workspace_packages()
        .iter()
        .find_map(|p| {
            if p.name == "quellcode" {
                Some(
                    p.manifest_path
                        .clone()
                        .parent()
                        .expect("Failed to get parent dir")
                        .to_path_buf()
                        .into_std_path_buf(),
                )
            } else {
                None
            }
        })
        .expect("Failed to find quellcode dir")
}

fn regenerate_bindings(args: Args) {
    let dir =
        quellcode_dir().join(var("TS_RS_EXPORT_DIR").expect("Failed to get TS_RS_EXPORT_DIR"));

    let stderr = console::Term::stderr();
    let now = std::time::Instant::now();

    if args.verbose {
        println!("Bindings directory: {}\n", dir.display().purple().italic());
    }

    let removing_binding_msg = format!("{} Removing existing bindings...", "[1/2]".dimmed().bold());

    stderr
        .write_line(&removing_binding_msg)
        .expect("Failed to write line");

    let pb = ProgressBar::new(read_dir(&dir).expect("Failed to read dir").count() as u64);
    pb.set_style(
        ProgressStyle::with_template("{bar:50} [{pos:>7}/{len:>7}] {msg}")
            .expect("Failed to set style"),
    );

    for entry in read_dir(dir).expect("Failed to read dir") {
        let entry = entry.expect("Failed to read entry");

        pb.set_message(format!(
            "Removing {}",
            style(entry.file_name().to_string_lossy()).magenta().bold()
        ));

        remove_file(entry.path()).expect("Failed to remove file");
        pb.inc(1);
    }

    pb.finish_and_clear();

    stderr.clear_last_lines(1).expect("Failed to clear line");

    stderr
        .write_line(&format!(
            "{removing_binding_msg} {}",
            format!("{}µs", now.elapsed().as_micros())
                .bright_purple()
                .bold(),
        ))
        .expect("Failed to write line");

    let binding_msg = format!("{} Regenerating bindings...", "[2/2]".dimmed().bold());

    stderr
        .write_line(&binding_msg)
        .expect("Failed to write line");

    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {spinner:.magenta.bold} Cargo output: {msg}",
        )
        .unwrap()
        .tick_strings(TICKS),
    );

    let mut child = process::Command::new("cargo")
        .arg("test")
        .args(["--package", "quellcode", "export_bindings"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cargo");

    let child_stderr = child.stderr.take().expect("Failed to get stdout");
    let child_stdout = child.stdout.take().expect("Failed to get stderr");

    let reader_stderr = BufReader::new(child_stderr);
    let reader_stdout = BufReader::new(child_stdout);

    let mut lines: Vec<String> = Vec::new();

    for line in reader_stderr.lines().chain(reader_stdout.lines()) {
        let line = line.expect("Failed to read line");
        lines.push(line.clone());

        let line = format!("\"{line}\"").dimmed().italic().to_string();

        pb.set_message(line);
    }

    pb.finish_and_clear();
    let status = child.wait().expect("Failed to wait for child");

    if !status.success() {
        panic!(
            "Failed to generate bindings\nCargo output:\n{}",
            lines.join("\n")
        );
    }

    stderr.clear_last_lines(1).expect("Failed to clear line");

    stderr
        .write_line(&format!(
            "{binding_msg} {}",
            format!("{:.2}s", now.elapsed().as_secs_f32())
                .bright_purple()
                .bold(),
        ))
        .expect("Failed to write line");

    println!("\n{}", "Bindings generated successfully!!".green().bold());
}
