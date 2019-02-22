use lazy_static::lazy_static;
use linux_uhid_tokio::{Bus, CreateParams, UHIDDevice};
use maplit::hashmap;
use pretty_assertions::assert_eq;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::panic;
use std::process::Command;
use std::thread;
use std::time::Duration;

// Keyboard Report Descriptor
const RDESC: [u8; 63] = [
    0x05, 0x01, 0x09, 0x06, 0xa1, 0x01, 0x05, 0x07, 0x19, 0xe0, 0x29, 0xe7, 0x15, 0x00, 0x25, 0x01,
    0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x95, 0x01, 0x75, 0x08, 0x81, 0x03, 0x95, 0x05, 0x75, 0x01,
    0x05, 0x08, 0x19, 0x01, 0x29, 0x05, 0x91, 0x02, 0x95, 0x01, 0x75, 0x03, 0x91, 0x03, 0x95, 0x06,
    0x75, 0x08, 0x15, 0x00, 0x25, 0x65, 0x05, 0x07, 0x19, 0x00, 0x29, 0x65, 0x81, 0x00, 0xc0,
];

const ALPHA_NUMERIC: &'static str =
    "1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const SYMBOLS: &'static str = "\"#!$%&'()*+,-.\\/:;<=>?@[]^_`{|}~\"";
//const EXPECTED_SYMBOLS: &'static str = "#!$%&'()*+,-./:;<=>?@\\[]^_`{|}~\"";

lazy_static! {
    static ref X_LAYOUT_MAP: HashMap<&'static str, (&'static str, Option<&'static str>)> = hashmap! {
        "LAYOUT_GERMAN" => ("de", None),
        "LAYOUT_PORTUGUESE_BRAZILIAN" => ("br", None),
        "LAYOUT_FRENCH" => ("fr", None),
        "LAYOUT_US_ENGLISH" => ("us", None),
        "LAYOUT_FINNISH" => ("fi", None),
        "LAYOUT_SPANISH_LATIN_AMERICA" => ("latam",  None),
        "LAYOUT_FRENCH_BELGIAN" => ("be", None),
        "LAYOUT_IRISH" => ("ie", None),
        "LAYOUT_SWEDISH" => ("se", None),
        "LAYOUT_GERMAN_SWISS" => ("ch", None),
        "LAYOUT_CANADIAN_FRENCH" => ("ca", Some("fr")),
        "LAYOUT_SPANISH" => ("es", None),
        "LAYOUT_PORTUGUESE" => ("pt", None),
        "LAYOUT_ICELANDIC" => ("is", None),
        "LAYOUT_TURKISH" => ("tr", None),
        "LAYOUT_US_INTERNATIONAL" => ("us", Some("intl")),
        "LAYOUT_CANADIAN_MULTILINGUAL" => ("ca", Some("multi")),
        "LAYOUT_FRENCH_SWISS" => ("ch", Some("fr")),
        "LAYOUT_DANISH" => ("dk", None),
        "LAYOUT_ITALIAN" => ("it", None),
        "LAYOUT_GERMAN_MAC" => ("de", Some("mac")),
        "LAYOUT_NORWEGIAN" => ("no", None),
        "LAYOUT_UNITED_KINGDOM" => ("gb", None),
    };
}

fn set_x_keyboard_layout(layout: &str, variant: Option<&str>) {
    let mut builder = Command::new("sudo");

    builder.args(&["localectl", "set-x11-keymap", layout]);
    eprintln!("Setting layout: {}", layout);

    if let Some(variant) = variant {
        builder.args(&["", variant]);
        eprintln!("Setting variant: {}", variant);
    }

    builder
        .output()
        .expect(&format!("Failed to set x keyboard layout for {}", layout));

    Command::new("sudo")
        .arg("setupcon")
        .output()
        .expect("Failed to setup console");
}

fn write_string_for_layout(string: &str, layout: &str) {
    let create_params = CreateParams {
        name: String::from("all_layouts_uhid"),
        phys: String::from(""),
        uniq: String::from(""),
        bus: Bus::USB,
        vendor: 0x15d9,
        product: 0x0a37,
        version: 0,
        country: 0,
        data: RDESC.to_vec(),
    };

    let core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let mut uhid_device = UHIDDevice::create(&handle, create_params, None).unwrap();
    let mut input = String::new();

    let packets =
        keyboard_scancodes::string_to_hid_packets(layout, &format!("{}\n", string)).unwrap();

    uhid_device.send_input(&[0u8; 8]).unwrap();

    thread::sleep(Duration::from_millis(100));
    // helps when debugging testing to wait on enter being pressed in console
    // std::io::stdin().read_line(&mut input).unwrap();

    for packet in packets.chunks(8) {
        uhid_device.send_input(&packet).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    uhid_device.destroy().unwrap();

    std::io::stdin().read_line(&mut input).unwrap();

    assert_eq!(
        input.trim(),
        string,
        "Unexpected output for layout: {}",
        layout
    );
}

fn run_layout_test<T>(layout: &str, test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    let (x_layout, x_variant) = X_LAYOUT_MAP.get(layout).unwrap();
    set_x_keyboard_layout(x_layout, *x_variant);

    let result = panic::catch_unwind(|| test());

    set_x_keyboard_layout("gb", None);

    assert!(result.is_ok())
}

#[test]
#[ignore]
fn create_uhid_device() {
    let create_params = CreateParams {
        name: String::from("all_layouts_uhid"),
        phys: String::from(""),
        uniq: String::from(""),
        bus: Bus::USB,
        vendor: 0x15d9,
        product: 0x0a37,
        version: 0,
        country: 0,
        data: RDESC.to_vec(),
    };

    let core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let mut uhid_device = UHIDDevice::create(&handle, create_params, None).unwrap();
    loop {}
}

macro_rules! test_layout {
    ($f:ident, $l:ident, $s:ident) => {
        #[test]
        fn $f() {
            run_layout_test(stringify!($l), || {
                write_string_for_layout($s, stringify!($l));
            });
        }
    };
}

test_layout!(
    test_alphanumeric_layout_german,
    LAYOUT_GERMAN,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_portuguese_brazilian,
    LAYOUT_PORTUGUESE_BRAZILIAN,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_french,
    LAYOUT_FRENCH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_us_english,
    LAYOUT_US_ENGLISH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_finnish,
    LAYOUT_FINNISH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_spanish_latin_america,
    LAYOUT_SPANISH_LATIN_AMERICA,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_french_belgian,
    LAYOUT_FRENCH_BELGIAN,
    ALPHA_NUMERIC
);
test_layout!(test_alphanumeric_layout_irish, LAYOUT_IRISH, ALPHA_NUMERIC);
test_layout!(
    test_alphanumeric_layout_swedish,
    LAYOUT_SWEDISH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_german_swiss,
    LAYOUT_GERMAN_SWISS,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_canadian_french,
    LAYOUT_CANADIAN_FRENCH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_spanish,
    LAYOUT_SPANISH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_portuguese,
    LAYOUT_PORTUGUESE,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_icelandic,
    LAYOUT_ICELANDIC,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_turkish,
    LAYOUT_TURKISH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_us_international,
    LAYOUT_US_INTERNATIONAL,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_canadian_multilingual,
    LAYOUT_CANADIAN_MULTILINGUAL,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_french_swiss,
    LAYOUT_FRENCH_SWISS,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_danish,
    LAYOUT_DANISH,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_italian,
    LAYOUT_ITALIAN,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_german_mac,
    LAYOUT_GERMAN_MAC,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_norwegian,
    LAYOUT_NORWEGIAN,
    ALPHA_NUMERIC
);
test_layout!(
    test_alphanumeric_layout_united_kingdom,
    LAYOUT_UNITED_KINGDOM,
    ALPHA_NUMERIC
);
test_layout!(test_symbols_layout_german, LAYOUT_GERMAN, SYMBOLS);
test_layout!(
    test_symbols_layout_portuguese_brazilian,
    LAYOUT_PORTUGUESE_BRAZILIAN,
    SYMBOLS
);
test_layout!(test_symbols_layout_french, LAYOUT_FRENCH, SYMBOLS);
test_layout!(test_symbols_layout_us_english, LAYOUT_US_ENGLISH, SYMBOLS);
test_layout!(test_symbols_layout_finnish, LAYOUT_FINNISH, SYMBOLS);
test_layout!(
    test_symbols_layout_spanish_latin_america,
    LAYOUT_SPANISH_LATIN_AMERICA,
    SYMBOLS
);
test_layout!(
    test_symbols_layout_french_belgian,
    LAYOUT_FRENCH_BELGIAN,
    SYMBOLS
);
test_layout!(test_symbols_layout_irish, LAYOUT_IRISH, SYMBOLS);
test_layout!(test_symbols_layout_swedish, LAYOUT_SWEDISH, SYMBOLS);
test_layout!(
    test_symbols_layout_german_swiss,
    LAYOUT_GERMAN_SWISS,
    SYMBOLS
);
test_layout!(
    test_symbols_layout_canadian_french,
    LAYOUT_CANADIAN_FRENCH,
    SYMBOLS
);
test_layout!(test_symbols_layout_spanish, LAYOUT_SPANISH, SYMBOLS);
test_layout!(test_symbols_layout_portuguese, LAYOUT_PORTUGUESE, SYMBOLS);
test_layout!(test_symbols_layout_icelandic, LAYOUT_ICELANDIC, SYMBOLS);
test_layout!(test_symbols_layout_turkish, LAYOUT_TURKISH, SYMBOLS);
test_layout!(
    test_symbols_layout_us_international,
    LAYOUT_US_INTERNATIONAL,
    SYMBOLS
);
test_layout!(
    test_symbols_layout_canadian_multilingual,
    LAYOUT_CANADIAN_MULTILINGUAL,
    SYMBOLS
);
test_layout!(
    test_symbols_layout_french_swiss,
    LAYOUT_FRENCH_SWISS,
    SYMBOLS
);
test_layout!(test_symbols_layout_danish, LAYOUT_DANISH, SYMBOLS);
test_layout!(test_symbols_layout_italian, LAYOUT_ITALIAN, SYMBOLS);
test_layout!(test_symbols_layout_german_mac, LAYOUT_GERMAN_MAC, SYMBOLS);
test_layout!(test_symbols_layout_norwegian, LAYOUT_NORWEGIAN, SYMBOLS);
test_layout!(
    test_symbols_layout_united_kingdom,
    LAYOUT_UNITED_KINGDOM,
    SYMBOLS
);
