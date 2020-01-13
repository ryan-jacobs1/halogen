#[macro_use]
extern crate clap;
use std::process::{Command, Stdio};

#[derive(Clap)]
#[clap(version = "1.0", author = "Ryan Jacobs")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "Build", version = "1.0", author = "Ryan Jacobs")]
    Build(Build),
    Run,
    Runner(Runner)
}

#[derive(Clap)]
struct Build {
    #[clap(short = "d")]
    debug: bool
}

#[derive(Clap)]
struct Runner {
    #[clap(required = true, min_values = 1)]
    path: String
}

fn main() {
    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Build(build) => handle_build(build),
        SubCommand::Run => handle_run(),
        SubCommand::Runner(_) => handle_runner()
    }
}

fn handle_build(build: Build) {

}

fn handle_run() {
    println!("Running oxos");
    Command::new("cargo")
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .arg("xrun")
    .spawn()
    .expect("Failed to run OxidizedOS");

}

fn handle_runner() {
    println!("Generating iso...");
    // Make isodir
    Command::new("mkdir")
    .args(&["-p", "isodir/boot/grub"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to crate isodir");

    // Copy binary
    Command::new("cp")
    .args(&["target/x86_64-oxos/debug/oxos", "isodir/boot/oxos.bin"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to copy oxos binary");

    // Copy grub cfg
    Command::new("cp")
    .args(&["grub.cfg", "isodir/boot/grub/grub.cfg"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to copy oxos binary");

    // Generate iso
    Command::new("grub-mkrescue")
    .args(&["-o", "oxos.iso", "isodir"])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to generate oxos iso");

    Command::new("qemu-system-x86_64")
    .args(&["-smp", "4", "-cdrom", "oxos.iso", "-nographic", "--monitor", "none"])
    .stdout(Stdio::inherit())
    .stdin(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("Failed to run qemu");
    
    
}