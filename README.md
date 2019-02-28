# Keyboard Layouts

Get the keycodes and modifier keys required to type an ASCII string for a number of different keyboard layouts. 

Takes inspiration and the [initial layout mappings](https://github.com/PaulStoffregen/cores/blob/master/teensy3/keylayouts.h) from the [Teensyduino project](https://github.com/PaulStoffregen/cores).

It works by preprocessing a C header file that describes the key mappings for each layout, including any deadkeys using `#define`'s. It then uses [bindgen](https://docs.rs/bindgen/0.47.2/bindgen/) to convert those into Rust constants and then [syn](https://docs.rs/syn/0.15.26/syn/) to extract the relevant keycodes and masks. It finally uses [quote!](https://docs.rs/quote/0.6.11/quote/) and [lazystatic!](https://docs.rs/lazy_static/1.2.0/lazy_static/) to produce a layout map enabling you to switch keyboard layouts on the fly without recompilation. 

## Example Usage

```rust

let test_string = "This is a test string.\n";

// Get the sequence of HID packets that would be produced by a keyboard with the specified layout
let hid_packets = keyboard_layouts::string_to_hid_packets("LAYOUT_UNITED_KINGDOM", test_string).unwrap();

// Write those HID packets to your virtual keyboard device. In this case a OTG HID gadget device file (linux).
std::fs::write("/dev/hidg0", hid_packets);
```

### Virtual Keyboard Device

This depends on your operating system and underlying hardware. So far this has only been tried on Linux but the HID packets should be valid for Windows and Mac.

On Linux you can either:
- Create a HID gadget device file on a Linux SBC with an OTG USB port. E.g. Raspberry Pi, Beaglebone. [This guide describes how](https://www.isticktoit.net/?p=1383)
- Check out the tests to see how to use the [tokio-linux-uhid](https://crates.io/crates/tokio-linux-uhid) crate to create a virtual HID device on a Linux desktop

I'm afraid for Windows and Mac I have no idea.

## kbsim CLI tool

There is a CLI tool, `kbsim`, included that can be useful.
```
USAGE:
    kbsim [FLAGS] [OPTIONS] [STRING]

FLAGS:
    -h, --help       Prints help information
    -n, --newline    Hit the 'Enter' key after writing the string
    -V, --version    Prints version information

OPTIONS:
    -c, --cooldown <cooldown>    Specify the number of milliseconds to wait between sending each HID packet to the
                                 device file [default: 0]
    -d, --delay <delay>          Specify the number of seconds to wait before writing [default: 0]
    -f, --hid-file <hid_file>    The HID file to write to. Defaults to /dev/hidg0
    -l, --layout <layout>        The keyboard layout to use. Specify 'list' to show all available layouts [default:
                                 LAYOUT_US_ENGLISH]

ARGS:
    <STRING>  
```

## Supported Layouts 

Spanish

Canadian French

German Mac

German Swiss

Icelandic

United Kingdom

Italian

French Swiss

Finnish

Danish

French

German

Turkish

French Belgian

Portuguese

Canadian Multilingual

Spanish Latin America

US English

US International

Swedish

Portuguese Brazilian

Irish

Norwegian

## Testing 

Testing all the layouts are correct is hard. As a result the tests are hacky.

Testing for each layout is split into alphanumeric and symbols.
Each test:
1. Sets the user session's keyboard layout (only in plain virtual console, no X)
1. Creates a virtual HID device on the machine using /dev/uhid (user needs permissions)
1. Writes all the specified characters to the virtual HID device (cursor needs to be in the testing terminal and stay there)
1. Reads the string of types from stdin and compares with the original.

