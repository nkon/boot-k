use blxlib::image_header::ImageHeader;
use blxlib::{crc32, image_header};
use getopts::Options;
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path;
use std::path::PathBuf;
use std::process::Command;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("bintool: bintool\nUsage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn run_info(in_file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("\n*** run_info ***\n");
    let mut file = File::open(in_file_path)?;
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf)?;

    let ih = image_header::load_from_buf(&buf[0..std::mem::size_of::<ImageHeader>()]);

    println!("header_magic: {:04x}", ih.header_magic);
    println!("header_length: {}", ih.header_length);
    println!("hv_major: {}", ih.hv_major);
    println!("hv_minor: {}", ih.hv_minor);
    println!("iv_major: {}", ih.iv_major);
    println!("iv_minor: {}", ih.iv_minor);
    println!("iv_patch: {}", ih.iv_patch);
    println!("iv_build: {:08x}", ih.iv_build);
    println!("image_length: {:04x}", ih.image_length);
    println!("payload_crc: {:04x}", ih.payload_crc);
    println!("crc32: {:04x}", ih.crc32);
    // println!("{:?}",ih);
    Ok(())
}

fn run_crc(in_file_path: &PathBuf, out_file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("\n*** run_crc ***\n");
    let mut in_file = File::open(in_file_path)?;
    let mut in_buf = Vec::<u8>::new();
    in_file.read_to_end(&mut in_buf)?;

    let header_len = std::mem::size_of::<ImageHeader>();

    let buf_ih = &in_buf[0..header_len];
    let buf_payload = &in_buf[header_len..];

    let mut ih = image_header::load_from_buf(buf_ih);

    ih.set_crc32();

    let mut out_file = File::create(out_file_path)?;
    out_file.write_all(image_header::as_bytes_with_len(&ih, header_len))?;
    out_file.write_all(buf_payload)?;

    Ok(())
}

fn run_sign(in_file_path: &PathBuf, out_file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("\n*** run_sign ***\n");
    let mut in_file = File::open(in_file_path)?;
    let mut in_buf = Vec::<u8>::new();
    in_file.read_to_end(&mut in_buf)?;

    let header_len = std::mem::size_of::<ImageHeader>();

    let buf_ih = &in_buf[0..header_len];
    let buf_payload = &in_buf[header_len..];
    let payload_length = buf_payload.len();

    let mut ih = image_header::load_from_buf(buf_ih);

    ih.payload_crc = crc32::crc32(buf_payload);
    ih.image_length = payload_length as u32;

    ih.set_crc32();

    let mut out_file = File::create(out_file_path)?;
    out_file.write_all(image_header::as_bytes_with_len(&ih, header_len))?;
    out_file.write_all(buf_payload)?;

    Ok(())
}

fn run_version(in_file_path: &PathBuf, out_file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("\n*** run_version ***\n");
    let mut in_file = File::open(in_file_path)?;
    let mut in_buf = Vec::<u8>::new();
    let _ = in_file.read_to_end(&mut in_buf)?;

    let header_len = std::mem::size_of::<ImageHeader>();

    let buf_ih = &in_buf[0..header_len];
    let buf_payload = &in_buf[header_len..];

    let mut ih = image_header::load_from_buf(buf_ih);

    let commit_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("failed to execute command \"git rev-parse HEAD\"");
    let commit_hash = String::from_utf8(commit_hash.stdout.to_vec())
        .unwrap()
        .replace('\n', "");
    println!("commit hash={}", commit_hash);
    let hash_regex = Regex::new(r"^(.{8})").unwrap();
    match hash_regex.captures(&commit_hash) {
        Some(caps) => {
            println!("build: {}", &caps[0]);
            ih.iv_build = u32::from_str_radix(&caps[0], 16)?;
        }
        None => println!("Not found"),
    }

    let pkg_info = Command::new("cargo")
        .args(["pkgid", "--manifest-path=../app-blinky/Cargo.toml"])
        .output()
        .expect("fail to execute command \"cargo pkgid --manifest-path=../app-blinky/Cargo.toml\"");
    let pkg_info = String::from_utf8(pkg_info.stdout.to_vec())
        .unwrap()
        .replace('\n', "");
    println!("pkg_info={}", pkg_info);
    let pkg_regex = Regex::new(r"(\d+)\.(\d+)\.(\d+)$").unwrap();
    match pkg_regex.captures(&pkg_info) {
        Some(caps) => {
            println!("major: {}", &caps[1]);
            println!("minor: {}", &caps[2]);
            println!("patch {}", &caps[3]);
            ih.iv_major = caps[1].parse::<u8>()?;
            ih.iv_minor = caps[2].parse::<u8>()?;
            ih.iv_patch = caps[3].parse::<u16>()?;
        }
        None => println!("Not found"),
    }

    ih.set_crc32();

    let mut out_file = File::create(out_file_path)?;
    out_file.write_all(image_header::as_bytes_with_len(&ih, header_len))?;
    out_file.write_all(buf_payload)?;

    Ok(())
}

fn run_all(in_file_path: &PathBuf, out_file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("\n*** run_version ***\n");
    let mut in_file = File::open(in_file_path)?;
    let mut in_buf = Vec::<u8>::new();
    let _ = in_file.read_to_end(&mut in_buf)?;

    let header_len = std::mem::size_of::<ImageHeader>();

    let buf_ih = &in_buf[0..header_len];
    let buf_payload = &in_buf[header_len..];
    let payload_length = buf_payload.len();

    let mut ih = image_header::load_from_buf(buf_ih);

    // update version
    let commit_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("failed to execute command \"git rev-parse HEAD\"");
    let commit_hash = String::from_utf8(commit_hash.stdout.to_vec())
        .unwrap()
        .replace('\n', "");
    println!("commit hash={}", commit_hash);
    let hash_regex = Regex::new(r"^(.{8})").unwrap();
    match hash_regex.captures(&commit_hash) {
        Some(caps) => {
            ih.iv_build = u32::from_str_radix(&caps[0], 16)?;
        }
        None => println!("Not found"),
    }

    let pkg_info = Command::new("cargo")
        .args(["pkgid", "--manifest-path=../app-blinky/Cargo.toml"])
        .output()
        .expect("fail to execute command \"cargo pkgid --manifest-path=../app-blinky/Cargo.toml\"");
    let pkg_info = String::from_utf8(pkg_info.stdout.to_vec())
        .unwrap()
        .replace('\n', "");
    println!("pkg_info={}", pkg_info);
    let pkg_regex = Regex::new(r"(\d+)\.(\d+)\.(\d+)$").unwrap();
    match pkg_regex.captures(&pkg_info) {
        Some(caps) => {
            ih.iv_major = caps[1].parse::<u8>()?;
            ih.iv_minor = caps[2].parse::<u8>()?;
            ih.iv_patch = caps[3].parse::<u16>()?;
        }
        None => println!("Not found"),
    }

    // update payload_crc
    ih.payload_crc = crc32::crc32(buf_payload);
    ih.image_length = payload_length as u32;

    // update header_crc
    ih.set_crc32();

    let mut out_file = File::create(out_file_path)?;
    out_file.write_all(image_header::as_bytes_with_len(&ih, header_len))?;
    out_file.write_all(buf_payload)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("c", "", "sub command", "sign|crc|version|all|info");
    opts.optopt("i", "", "input file", "INFILE");
    opts.optopt("o", "", "output file", "OUTFILE");

    match opts.parse(&args[1..]) {
        Ok(matches) => {
            if matches.opt_present("h") {
                print_usage(&program, opts);
                std::process::exit(0);
            }

            let mut in_file_path = path::Path::new("").to_path_buf();
            if let Some(in_file_str) = matches.opt_str("i") {
                in_file_path = path::Path::new(&in_file_str).to_path_buf();
                if !in_file_path.exists() {
                    eprintln!("file not found {}", in_file_path.to_str().unwrap());
                    std::process::exit(1);
                }
            }

            let mut out_file_path = path::Path::new("").to_path_buf();
            if let Some(out_file_str) = matches.opt_str("o") {
                out_file_path = path::Path::new(&out_file_str).to_path_buf();
            }

            if let Some(command_str) = matches.opt_str("c") {
                println!("command={}", command_str);
                println!("in_file_path={}", in_file_path.to_string_lossy());
                println!("out_file_path={}", out_file_path.to_string_lossy());
                match &*command_str {
                    "info" => {
                        run_info(&in_file_path).unwrap();
                    }
                    "crc" => {
                        run_crc(&in_file_path, &out_file_path).unwrap();
                    }
                    "sign" => {
                        run_sign(&in_file_path, &out_file_path).unwrap();
                    }
                    "version" => {
                        run_version(&in_file_path, &out_file_path).unwrap();
                    }
                    "all" => {
                        run_all(&in_file_path, &out_file_path).unwrap();
                    }
                    _ => {
                        print_usage(&program, opts);
                        std::process::exit(1);
                    }
                }
            } else {
                print_usage(&program, opts);
                std::process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(0);
        }
    }
}
