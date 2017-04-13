/*
 * A simple program that aims to extract various string constants from a JAR file (Java)
 */

extern crate classreader;
extern crate zip;
extern crate docopt;
extern crate rustc_serialize;
extern crate java_properties;

use docopt::Docopt;
use classreader::*;
use std::fs::File;
use java_properties::PropertiesIter;
use std::io::BufReader;

const USAGE: &'static str = "
Java Strings extractor.

Usage:
  jstrings <source>...
  javaminer (-h | --help)

Options:
  -h --help     Show this screen.

Source can be one or more class or jar files.
";

#[derive(Debug, RustcDecodable)]
struct Args {
	arg_source: Vec<String>
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    for f in args.arg_source {
        if f.ends_with(".class") {
            process_class_file(&f);
        } else if f.ends_with(".jar") {
            process_jar_file(&f);
        } else if f.ends_with(".properties") {
            process_properties_file(&f);
		}
    }
}

fn process_jar_file(file_name: &String) {
    let file = File::open(file_name).expect("couldn't find a file!");
    let mut zip = zip::ZipArchive::new(file).expect("could not read JAR");
    for i in 0..zip.len() {
        let mut class_file = zip.by_index(i).unwrap();
        if class_file.name().ends_with(".class") {
            let class = ClassReader::new_from_reader(&mut class_file).unwrap();
            process_class(&class);
        } else if class_file.name().ends_with(".properties") {
			process_properties(class_file);
		}
    }
}

fn process_class_file(file_name: &String) {
    let class = ClassReader::new_from_path(&file_name).unwrap();
    process_class(&class);
}

fn process_class(class: &Class) {
    assert_eq!(0xCAFEBABE, class.magic);
    for jstr in &class.constant_pool {
		match jstr {
			&ConstantPoolInfo::String(index) => {
				println!("{}", get_string(&class, index as usize));
			},
			_ => {}
		}
	}
}

/// Get constant from a pool, correcting for java's 1-based indexes.
fn get_const(class: &Class, i: usize) -> &ConstantPoolInfo {
    &class.constant_pool[i - 1]
}

/// Get string from constant pool
fn get_string(class: &Class, index: usize) -> String {
    match get_const(class, index) {
        &ConstantPoolInfo::Utf8(ref s) => s.clone(),
        _ => "?".to_string()
    }
}

fn process_properties_file(file_name: &String) {
	let f = File::open(file_name).expect("couldn't find a file!");
	process_properties(BufReader::new(f));
}

fn process_properties<R: std::io::Read>(f: R) {
    PropertiesIter::new(f).read_into(|_, v| {
      println!("{}", v);
    }).expect("failed to read a properties file");
}
