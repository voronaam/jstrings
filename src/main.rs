/*
 * A simple program that aims to extract various string constants from a JAR file (Java)
 */

extern crate classreader;
extern crate zip;
extern crate docopt;
extern crate serde;
extern crate java_properties;
extern crate shannon_entropy;

use docopt::Docopt;
use serde::Deserialize;
use classreader::*;
use std::fs::File;
use java_properties::PropertiesIter;
use std::io::BufReader;
use shannon_entropy::shannon_entropy;

const USAGE: &'static str = "
Java Strings extractor.

Usage:
  jstrings [-e] <source>...
  jstrings [-t <threshold>] <source>...
  jstrings (-h | --help)

Options:
  -e              Print average entropy of each string (average of entropy of each word in the string)
  -t <threshold>  Do not print string with entropy below threshold. Implies entropy computation.
  -h --help       Show this screen.

Source can be one or more class or jar files.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_e: bool,
    flag_t: Option<f32>,
    arg_source: Vec<String>
}

type Printer = Box<dyn Fn(&str)>;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
    let printer = printer_factory(&args);
    for f in args.arg_source {
        if f.ends_with(".class") {
            process_class_file(&f, &printer);
        } else if f.ends_with(".jar") {
            process_jar_file(&f, &printer);
        } else if f.ends_with(".properties") {
            process_properties_file(&f, &printer);
        }
    }
}

fn process_jar_file(file_name: &str, printer: &Printer) {
    let file = File::open(file_name).expect("couldn't find a file!");
    let mut zip = zip::ZipArchive::new(file).expect("could not read JAR");
    for i in 0..zip.len() {
        let mut class_file = zip.by_index(i).unwrap();
        if class_file.name().ends_with(".class") {
			if let Ok(class) = ClassReader::new_from_reader(&mut class_file) {
				process_class(&class, printer);
			}
        } else if class_file.name().ends_with(".properties") {
            process_properties(class_file, printer);
        }
    }
}

fn process_class_file(file_name: &str, printer: &Printer) {
    let class = ClassReader::new_from_path(file_name).unwrap();
    process_class(&class, printer);
}

fn process_class(class: &Class, printer: &Printer) {
    assert_eq!(0xCAFEBABE, class.magic);
    for jstr in &class.constant_pool {
		if let ConstantPoolInfo::String(index) = *jstr {
			printer(get_string(class, index as usize));
		}
    }
}

/// Get constant from a pool, correcting for java's 1-based indexes.
fn get_const(class: &Class, i: usize) -> &ConstantPoolInfo {
    &class.constant_pool[i - 1]
}

/// Get string from constant pool
fn get_string(class: &Class, index: usize) -> &str {
    match *get_const(class, index) {
        ConstantPoolInfo::Utf8(ref s) => s,
        _ => "?"
    }
}

fn process_properties_file(file_name: &str, printer: &Printer) {
    let f = File::open(file_name).expect("couldn't find a file!");
    process_properties(BufReader::new(f), printer);
}

fn process_properties<R: std::io::Read>(f: R, printer: &Printer) {
    PropertiesIter::new(f).read_into(|_, v| {
      printer(&v);
    }).expect("failed to read a properties file");
}

// Output variants
fn printer_factory(args: &Args) -> Printer {
	if let Some(t) = args.flag_t {
		return Box::new(move |s: &str| {
				let e = average_entropy(s);
				if e >= t {
					println!("{:>6.2} {}", e, s)
				}
			})
	}
    if args.flag_e {
        return Box::new(print_entropy)
    }
    Box::new(print_only)
}

fn print_only(s: &str) {
    println!("{}", s)
}

fn print_entropy(s: &str) {
    println!("{:>6.2} {}", average_entropy(s), s)
}

fn average_entropy(s: &str) -> f32 {
    let tuple = s.split_whitespace().map(shannon_entropy).fold( (0.0, 0),
      |acc, w| (acc.0 + w, acc.1 + 1));
    tuple.0 / tuple.1 as f32
}
