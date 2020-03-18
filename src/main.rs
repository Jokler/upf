use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use structopt::StructOpt;
use upf::UploaderTemplate;

#[derive(Debug, StructOpt)]
#[structopt(about = "An upload program to simplify using file sharing services")]
struct Args {
    /// Template to use for the upload
    template: String,
    /// File to upload
    file: Option<PathBuf>,
    /// Filename to upload
    #[structopt(short, long)]
    file_name: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let path = match find_config_dir() {
        Some(p) => p,
        None => {
            eprintln!("Failed to find a valid config directory");
            return;
        }
    };

    let mut path = path.join(args.template);
    if path.extension() != Some(OsStr::new("toml")) {
        path.set_extension("toml");
    }
    let template = match UploaderTemplate::from_file(path) {
        Ok(t) => t,
        Err(e) => {
            print_error(&Box::new(e));
            return;
        }
    };

    let mut file_name: Option<String> = None;
    let mut data = Vec::new();
    if let Some(path) = args.file {
        file_name = path
            .file_name()
            .and_then(|o| o.to_str().map(|s| s.to_owned()));
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                return;
            }
        };
        let mut buf_reader = BufReader::new(file);
        if let Err(e) = buf_reader.read_to_end(&mut data) {
            eprintln!("Failed to read file: {}", e);
            return;
        }
    } else if atty::isnt(atty::Stream::Stdin) {
        if let Err(e) = std::io::stdin().read_to_end(&mut data) {
            eprintln!("An error occurred while reading from stdin: {}", e);
            return;
        }
        if args.file_name.is_none() {
            file_name = Some(String::from("stdin"));
        }
    } else {
        eprintln!("Supply either a file path or pipe data into stdin");
        return;
    }

    if args.file_name.is_some() {
        file_name = args.file_name;
    }
    match upf::upload(template, data, file_name).await {
        Ok(resp) => {
            println!("URL: {}", resp.url);
            if let Some(thumb) = resp.thumbnail_url {
                println!("Thumbnail URL: {}", thumb);
            }
            if let Some(del) = resp.deletion_url {
                println!("Deletion URL: {}", del);
            }
        }
        Err(e) => print_error(&Box::new(e)),
    }
}

fn print_error(mut e: &dyn Error) {
    let mut text = format!("{}", e);
    while let Some(err) = e.source() {
        text = format!("{}: {}", text, err);
        e = err;
    }
    eprintln!("{}", text);
}

//1. ~/.config/upf/config (or $XDG_CONFIG_HOME/upf/config if set)
//2. /etc/xdg/upf/config (or $XDG_CONFIG_DIRS/upf/config if set)
//3. /etc/upf/config
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
