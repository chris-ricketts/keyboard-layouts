use structopt::StructOpt;

use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::thread;
use std::time::Duration;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hid-keysim",
    about = "Simulates a HID keyboard, supporting multiple layouts."
)]
enum CliOpt {
    #[structopt(name = "show-layouts", help = "List the supported keyboard layouts")]
    List,
    #[structopt(name = "write", help = "Write a string by impersonating a keyboard")]
    Write {
        #[structopt(
            long = "hid-file",
            short = "f",
            help = "The HID file to write to. Defaults to /dev/hidg0"
        )]
        hid_file: Option<String>,
        #[structopt(
            long = "layout",
            short = "l",
            help = "The keyboard layout to use. Defaults to 'LAYOUT_US_ENGLISH'"
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
        #[structopt(
            long = "cooldown",
            short = "c",
            help = "Specify the number of milliseconds to wait between sending each packet",
            default_value = "0"
        )]
        cooldown: u64,
        #[structopt(name = "STRING")]
        string: String,
    },
    #[structopt(
        name = "show-keys",
        help = "Lists the modifiers and keys used to type each character in the given string"
    )]
    Keys {
        #[structopt(
            long = "layout",
            short = "l",
            help = "The keyboard layout to use. Defaults to 'LAYOUT_US_ENGLISH'"
        )]
        layout: Option<String>,
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
            cooldown,
            mut string,
        } => {
            let hid_file = hid_file.unwrap_or_else(|| "/dev/hidg0".to_string());

            let layout = layout.unwrap_or_else(|| "LAYOUT_US_ENGLISH".to_string());

            if newline {
                string.push('\n');
            }

            let hid_bytes = keyboard_scancodes::string_to_hid_packets(layout, string)
                .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))?;

            thread::sleep(Duration::from_secs(delay));

            for packet in hid_bytes.chunks(keyboard_scancodes::HID_PACKET_LEN) {
                fs::write(&hid_file, packet)?;
                thread::sleep(Duration::from_millis(cooldown));
            }

            Ok(())
        }
        CliOpt::Keys { layout, string } => {
            let layout = layout.unwrap_or_else(|| "LAYOUT_US_ENGLISH".to_string());

            let keys_and_modifiers =
                keyboard_scancodes::string_to_keys_and_modifiers(layout, string)
                    .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))?;

            for (i, (k, m)) in keys_and_modifiers.iter().enumerate() {
                // Don't print key releases
                if i % 2 == 0 {
                    println!("Modifier: {} Key: {}", m, k);
                }
            }
            Ok(())
        }
    }
}
