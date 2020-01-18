#[macro_use]
extern crate clap;
use std::process::{Command, Stdio};

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
    Run,
    #[clap(name = "runner", version = "1.0", author = "Ryan Jacobs")]
    Runner(Runner),
    #[clap(name = "clean", version = "1.0", author = "Ryan Jacobs")]
    Clean,
}

#[derive(Clap)]
struct Build {
    #[clap(short = "d")]
    debug: bool
}

#[derive(Clap)]
struct Runner {
    #[clap(short = "p", long = "path")]
    path: String
}

fn main() {
    let opts = Opts::parse();
    let exit_code = match opts.subcmd {
        SubCommand::Build(build) => handle_build(build),
        SubCommand::Run => handle_run(),
        SubCommand::Runner(runner) => handle_runner(runner),
        SubCommand::Clean => handle_clean(),
    };
    std::process::exit(exit_code);
}

fn handle_build(build: Build) -> i32 {
    return 0;
}

fn handle_run() -> i32 {
    println!("Running oxos");
    Command::new("cargo")
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .arg("xrun")
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

    let status = Command::new("qemu-system-x86_64")
    .args(&["-smp", "4", "-cdrom", "oxos.iso", "-nographic", "--monitor", "none", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"])
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