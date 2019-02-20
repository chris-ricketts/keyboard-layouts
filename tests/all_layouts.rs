use linux_uhid_tokio::{Bus, CreateParams, UHIDDevice};
use maplit::hashmap;
use pretty_assertions::assert_eq;

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

// Had to fiddle to make this work. Between shell escaping chars and uhid packet loss its a
// nightmare
const SUPPORTED_ASCII: &'static str = "1023456789\\#\\!\\$\\%&\'()*+,-./:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\"";

fn set_x_keyboard_layout(layout: &str, variant: Option<&str>) {
    let mut builder = Command::new("setxkbmap");

    builder.args(&["-layout", layout]);

    if let Some(variant) = variant {
        builder.args(&["-variant", variant]);
    }

    builder
        .output()
        .expect(&format!("Failed to set x keyboard layout for {}", layout));
}

#[test]
fn all_layouts() {
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

    let x_layout_map = hashmap! {
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
        "LAYOUT_CANADIAN_FRENCH" => ("cf", None),
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

    let core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let mut uhid_device = UHIDDevice::create(&handle, create_params, None).unwrap();

    let mut input = String::new();

    for (layout, _) in keyboard_scancodes::available_layouts() {
        let (xlayout, xvariant) = x_layout_map.get(layout).unwrap();

        set_x_keyboard_layout(xlayout, *xvariant);

        let packets =
            keyboard_scancodes::string_to_hid_packets(layout, &format!("\"{}\n", SUPPORTED_ASCII))
                .unwrap();

        for packet in packets.chunks(8) {
            uhid_device.send_input(&packet).unwrap();
            thread::sleep(Duration::from_millis(50));
        }

        std::io::stdin().read_line(&mut input).unwrap();

        assert_eq!(
            input.trim(),
            SUPPORTED_ASCII,
            "Unexpected output for layout: {}",
            layout
        );
    }

    set_x_keyboard_layout("gb", None);
}
