//! rmpl - YAML-based monorepo build tool
//! Phase 1: Minimal bootstrap with no external dependencies

mod manifest;
mod workspace;
mod build;
mod resolver;
mod install;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        print_usage();
        process::exit(0);
    }
    
    let command = &args[0];
    let profile = args.get(1).map(|s| s.as_str()).unwrap_or("debug");
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    
    match command.as_str() {
        "build" => {
            if let Err(e) = build::build_workspace_with_profile(profile) {
                eprintln!("Build error: {}", e);
                process::exit(1);
            }
        }
        "install" => {
            if let Err(e) = install::install_workspace(profile, force) {
                eprintln!("Install error: {}", e);
                process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("rmpl - YAML-based monorepo build tool");
    println!();
    println!("Usage: rmpl <command> [options]");
    println!();
    println!("Commands:");
    println!("  build [debug|release]  Build the workspace (default: debug)");
    println!("  install [debug|release]  Install binaries to ~/.rmpl/bin");
    println!("                         Options: --force, -f");
    println!("  help                   Show this help message");
}
