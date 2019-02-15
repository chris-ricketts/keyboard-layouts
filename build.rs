use lazy_static::lazy_static;
use regex::RegexBuilder;
use std::env;
use std::path::PathBuf;
use syn::{Expr, Ident, Item, Lit};

const KEY_LAYOUTS_HEADER: &'static str = include_str!("keylayouts.h");

fn main() {
    let layouts = find_layout_definitions();

    get_keycodes_for_layout(layouts[0]);
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings!");
    panic!()
}

fn get_keycodes_for_layout(layout: &str) {
    let header_name = format!("{}.h", layout);
    let defined_layout = format!("#define {}\n{}", layout, KEY_LAYOUTS_HEADER);
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header_contents(&header_name, &defined_layout)
        .generate()
        .expect("Unable to generate bindings")
        .to_string();

    let syntax = syn::parse_str::<syn::File>(&bindings).expect("Failed to parse bindings");

    let keycode_mask: u16 = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "KEYCODE_MASK"))
        .expect("Failed to find KEYCODE_MASK");

    dbg!(keycode_mask);

    let keycodes: Vec<u16> = syntax
        .items
        .iter()
        .filter_map(|item| find_const_u16_with_name_containing(item, "ASCII_"))
        .map(|k| k & keycode_mask)
        .collect();

    dbg!(keycodes);
}

fn find_const_u16_with_name_containing(item: &Item, name: &str) -> Option<u16> {
    match item {
        Item::Const(c) => {
            let ident = c.ident.to_string();
            if ident.contains(name) {
                if let Expr::Lit(ref lit) = *c.expr {
                    if let Lit::Int(ref i) = lit.lit {
                        Some(i.value() as u16)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn find_layout_definitions() -> Vec<&'static str> {
    let layout_def_regex = RegexBuilder::new(r"//\#define\s+?(LAYOUT_.*)")
        .multi_line(true)
        .build()
        .expect("Failed to build regex");

    layout_def_regex
        .captures_iter(KEY_LAYOUTS_HEADER)
        .map(|cap| cap.get(1).unwrap().as_str())
        .collect()
}
