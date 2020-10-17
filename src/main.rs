use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read};
use std::path::PathBuf;

use anyhow::{anyhow, Result, bail};
use structopt::StructOpt;
use upf::UploaderTemplate;

#[derive(Debug, StructOpt)]
#[structopt(about = "An upload program to simplify using file sharing services")]
struct Args {
    /// Overwrite filename used on upload
    #[structopt(short, long)]
    file_name: Option<String>,
    /// Append output to the specified file
    #[structopt(short, long)]
    output: Option<PathBuf>,
    /// Print additional information
    #[structopt(short, long)]
    debug: bool,
    /// Template to use for the upload
    template: String,
    /// File to upload
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprint!("ERROR: {}", e);
        e.chain()
            .skip(1)
            .for_each(|cause| eprint!(": {}", cause));
        eprintln!();
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::from_args();

    let mut path = find_config_dir().ok_or(anyhow!("Failed to find a valid config directory"))?;

    path = path.join("templates/");
    path = path.join(args.template);
    if path.extension() != Some(OsStr::new("toml")) {
        path.set_extension("toml");
    }
    let template = UploaderTemplate::from_file(path)?;

    let mut file_name: Option<String> = None;
    let mut data = Vec::new();
    if let Some(path) = args.file {
        file_name = path
            .file_name()
            .and_then(|o| o.to_str().map(|s| s.to_owned()));
        let file = File::open(path).map_err(|e| anyhow!("Failed to open file: {}", e))?;
        let mut buf_reader = BufReader::new(file);
        buf_reader
            .read_to_end(&mut data)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;
    } else if atty::isnt(atty::Stream::Stdin) {
        std::io::stdin()
            .read_to_end(&mut data)
            .map_err(|e| anyhow!("An error occurred while reading from stdin: {}", e))?;
        if args.file_name.is_none() {
            file_name = Some(String::from("stdin"));
        }
    } else {
        bail!("Supply either a file path or pipe data into stdin");
    }

    if args.file_name.is_some() {
        file_name = args.file_name;
    }
    let resp = upf::upload(template, data, file_name, args.debug).await?;

    print!("{}", resp.url);
    // Ensure this gets printed first
    use std::io::Write;
    if let Err(e) = std::io::stdout().lock().flush() {
        eprintln!("Failed to flush stdout: {}", e);
    }

    eprintln!();
    for (name, url) in &resp.additional_urls {
        eprintln!("{}: {}", name, url);
    }

    if let Some(path) = args.output {
        if let Err(e) = (|| {
            let mut file = OpenOptions::new().append(true).create(true).open(path)?;

            file.write_all(format!("\nURL: {}\n", resp.url).as_bytes())?;
            for (name, url) in &resp.additional_urls {
                file.write_all(format!("{}: {}\n", name, url).as_bytes())?;
            }

            Ok::<(), std::io::Error>(())
        })() {
            bail!("Failed to write to output file: {}", e);
        }
    }

    Ok(())
}

//1. ~/.config/upf/ (or $XDG_CONFIG_HOME/upf/ if set)
//2. /etc/xdg/upf/ (or $XDG_CONFIG_DIRS/upf/ if set)
//3. /etc/upf/
fn find_config_dir() -> Option<PathBuf> {
    let mut config_home = std::env::var("XDG_CONFIG_HOME").ok().map(PathBuf::from);
    if config_home.is_none() {
        if let Some(home) = std::env::var("HOME").ok() {
            config_home = Some(PathBuf::from(home).join(".config/"));
        }
    }

    if let Some(config_home) = config_home {
        let path = config_home.join("upf/");
        if path.exists() {
            return Some(path);
        }
    }

    let mut config_dirs = std::env::var("XDG_CONFIG_DIRS").ok().map(PathBuf::from);
    if config_dirs.is_none() {
        config_dirs = Some(PathBuf::from("/etc/xdg/"));
    }

    if let Some(config_dirs) = config_dirs {
        let path = config_dirs.join("upf/");
        if path.exists() {
            return Some(path);
        }
    }

    let path = PathBuf::from("/etc/upf/");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
