use {std::io::Result, structopt::StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "strokes",
    about = "Shows what keys and modifiers will be required to type each character in the given string according to the layout"
)]
struct CliOpt {
    #[structopt(
        long = "layout",
        short = "l",
        help = "The keyboard layout to use. Specify 'list' to show all available layouts",
        default_value = "LAYOUT_US_ENGLISH"
    )]
    layout: String,
    #[structopt(name = "STRING")]
    string: Option<String>,
}

fn main() -> Result<()> {
    let CliOpt { layout, string } = CliOpt::from_args();

    if layout.to_lowercase() == "list" {
        for l in keyboard_layouts::available_layouts() {
            println!("{}", l);
        }
        return Ok(());
    }

    if let Some(string) = string {
        println!("Layout: {}", layout);
        println!("Keys and Modifiers to type: {}", &string);
        for key_mod in keyboard_layouts::string_to_keys_and_modifiers(&layout, &string).unwrap() {
            println!(
                "Key: {:#02X} Modifier: {:#02X}",
                key_mod.key, key_mod.modifier
            );
        }
    }

    Ok(())
}
