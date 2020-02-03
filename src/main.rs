#[macro_use]
extern crate clap;
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::time::SystemTime;

pub static EXIT_QEMU_SUCCESS: i64 = (5 << 1) | 1;
pub static EXIT_QEMU_SIGNAL: i64 = (4 << 1) | 1;


#[derive(Clap)]
#[clap(version = "1.0", author = "Ryan Jacobs")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "build", version = "1.0", author = "Ryan Jacobs")]
    Build(Build),
    #[clap(name = "run", version = "1.0", author = "Ryan Jacobs")]
    Run(Run),
    #[clap(name = "runner", version = "1.0", author = "Ryan Jacobs")]
    Runner(Runner),
    #[clap(name = "mksfs", version = "1.0", author = "Chris DeLaGarza")]
    Mksfs(Mksfs),
    #[clap(name = "clean", version = "1.0", author = "Ryan Jacobs")]
    Clean,
}

#[derive(Clap)]
struct Mksfs {
    #[clap(short = "n", long = "name")]
    name: String,
}

#[derive(Clap)]
struct Build {
    #[clap(short = "d")]
    debug: bool
}

#[derive(Clap)]
struct Run {
    #[clap(short, long)]
    graphic: bool
}

#[derive(Clap)]
struct Runner {
    #[clap(short = "p", long = "path")]
    path: String,
    #[clap(short, long)]
    graphic: bool
}

fn main() {
    let opts = Opts::parse();
    let exit_code = match opts.subcmd {
        SubCommand::Build(build) => handle_build(build),
        SubCommand::Run(run) => handle_run(run),
        SubCommand::Runner(runner) => handle_runner(runner),
        SubCommand::Mksfs(mksfs) => handle_mksfs(mksfs),
        SubCommand::Clean => handle_clean(),
    };
    std::process::exit(exit_code);
}

fn handle_mksfs(mksfs: Mksfs) -> i32 {
    let mut image = File::create("sfs.img").unwrap();
    let sector_size = 512; // 512 Bytes per sector
    let media_size = 1440 * 1024; // 1440 KB (Size of the filesystem)

    assert_eq!(0, media_size % sector_size);
    let block_size: u8 = (sector_size >> 8) as u8;
    let magic_num: Vec<u8> = vec![0x53, 0x46, 0x53];
    let total_blocks: u64 = media_size / sector_size;
    let reserved_blocks: u32 = 5;
    let version_number: u8 = 0x10;
    let index_area_size: u64 = 128; // Stored in Bytes
    let data_area_size: u64 = 0;    // Stored in Blocks
    let time_stamp = get_timestamp();
    let checksum: u8 = 0xFF & (magic_num[0] as u64 + magic_num[1] as u64 + magic_num[2] as u64 + version_number as u64 + (total_blocks >> 6) + (total_blocks >> 4) + (total_blocks >> 2) + total_blocks + (reserved_blocks >> 2) as u64 + (reserved_blocks >> 6) as u64) as u8;

    image.set_len(media_size).unwrap();
    image.seek(SeekFrom::Start(0x0194)).unwrap();
    image.write(&time_stamp.to_le_bytes()).unwrap();
    image.write(&data_area_size.to_le_bytes()).unwrap();
    image.write(&index_area_size.to_le_bytes()).unwrap();
    image.write(&magic_num).unwrap();
    image.write(&version_number.to_le_bytes()).unwrap();
    image.write(&total_blocks.to_le_bytes()).unwrap();
    image.write(&reserved_blocks.to_le_bytes()).unwrap();
    image.write(&block_size.to_le_bytes()).unwrap();
    image.write(&checksum.to_le_bytes()).unwrap();

    let volume_identifier = new_index_volume();
    let starter_marker = new_starter_marker();

    image.seek(SeekFrom::End(-128)).unwrap();
    image.write(starter_marker.as_slice()).unwrap();
    image.write(volume_identifier.as_slice()).unwrap();

    0
}

fn new_starter_marker() -> Vec<u8> {
    let mut buffer = Vec::with_capacity(64);
    let mut entry_type = vec![0x02 as u8];

    buffer.append(&mut entry_type);
    buffer.append(&mut vec![0x10; 63]);
    buffer
}

fn new_index_volume() -> Vec<u8> {
    let mut buffer = Vec::with_capacity(64);
    let mut entry_type = vec![0x01 as u8];
    let mut unused_or_reserved = vec![0 as u8; 3];
    let mut time_stamp: Vec<u8> = (get_timestamp().to_le_bytes()).to_vec();
    let mut volume_name: Vec<u8> = Vec::with_capacity(52);
    volume_name.append(&mut String::from("Volume Identifier").into_bytes());
    volume_name.append(&mut vec![0 as u8]);

    buffer.append(&mut entry_type);
    buffer.append(&mut unused_or_reserved);
    buffer.append(&mut time_stamp);
    buffer.append(&mut volume_name);
    buffer
}

fn get_timestamp() -> u64 {
    (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() / 15259 / 65536) as u64
}

fn handle_build(build: Build) -> i32 {
    return 0;
}

fn handle_run(run: Run) -> i32 {
    let mut args = vec!("xrun", "--");
    if run.graphic {
        args.push("-g")
    }
    println!("Running oxos");
    Command::new("cargo")
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .args(&args)
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to run OxidizedOS");
    
    return 0;
}

fn handle_runner(runner: Runner) -> i32 {
    println!("Generating iso...");
    // Make isodir
    Command::new("mkdir")
    .args(&["-p", "isodir/boot/grub"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to crate isodir");

    // Copy binary
    let input = runner.path;
    let output = "isodir/boot/oxos.bin";
    Command::new("cp")
    .args(&[&input, output])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to copy oxos binary");

    // Copy grub cfg
    Command::new("cp")
    .args(&["grub.cfg", "isodir/boot/grub/grub.cfg"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to copy oxos binary");

    // Generate iso
    Command::new("grub-mkrescue")
    .args(&["-o", "oxos.iso", "isodir"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to generate oxos iso");
    let mut args = vec!("-smp", "4", "-cdrom", "oxos.iso", "--monitor", "none", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio", "-drive", "file=sample.img,index=3,media=disk,format=raw", "-drive", "file=sfs.img,index=4,media=disk,format=raw");
    if !runner.graphic {
        args.push("-nographic");
    }
    let status = Command::new("qemu-system-x86_64")
    .args(&args)
    .stdout(Stdio::inherit())
    .stdin(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to run qemu");

    let code: i64 = match status.code() {
        Some(code) => code as i64,
        None => EXIT_QEMU_SIGNAL
    };

    match code {
        code if code == EXIT_QEMU_SUCCESS => 0,
        _ => 1
    }
    
}

pub fn handle_clean() -> i32 {

    Command::new("cargo")
    .args(&["clean"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to run cargo clean");

    Command::new("rm")
    .args(&["-rf", "isodir"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to remove isodir");

    Command::new("rm")
    .args(&["-f", "oxos.iso"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to start")
    .wait()
    .expect("Failed to remove isodir");

    return 0;
}