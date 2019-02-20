use structopt::StructOpt;

use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hid-keysim",
    about = "Simulates a HID keyboard, supporting multiple layouts."
)]
enum CliOpt {
    #[structopt(name = "show-layouts", help = "List the available layouts")]
    List,
    #[structopt(name = "write", help = "Write a string by impersonating a keyboard")]
    Write {
        #[structopt(
            long = "hid-file",
            short = "f",
            help = "The HID file to write to. Defaults to /dev/hidg0",
            parse(from_os_str)
        )]
        hid_file: Option<PathBuf>,
        #[structopt(
            long = "layout",
            short = "l",
            help = "The keyboard to use. Defaults to 'LAYOUT_US_ENGLISH'"
        )]
        layout: Option<String>,
        #[structopt(
            long = "newline",
            short = "n",
            help = "Hit the 'Enter' key after writing the string"
        )]
        newline: bool,
        #[structopt(
            long = "delay",
            short = "d",
            help = "Specify the number of seconds to wait before writing",
            default_value = "0"
        )]
        delay: u64,
        #[structopt(name = "STRING")]
        string: String,
    },
}

fn main() -> Result<()> {
    let cli_opt = CliOpt::from_args();

    match cli_opt {
        CliOpt::List => {
            for (key, _) in keyboard_scancodes::available_layouts() {
                println!("{}", key);
            }
            Ok(())
        }
        CliOpt::Write {
            hid_file,
            layout,
            newline,
            delay,
            mut string,
        } => {
            let hid_file = hid_file.unwrap_or_else(|| PathBuf::from("/dev/hidg0"));

            let layout = layout.unwrap_or_else(|| "LAYOUT_US_ENGLISH".to_string());

            if newline {
                string.push('\n');
            }

            let hid_bytes = keyboard_scancodes::string_to_hid_packets(layout, string)
                .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))?;

            thread::sleep(Duration::from_secs(delay));

            fs::write(hid_file, hid_bytes)
        }
    }
}
