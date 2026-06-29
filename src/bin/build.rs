//! Build orchestrator - invokes rustc to compile packages

use crate::resolver::ResolvedWorkspace;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn build_workspace() -> Result<(), String> {
    build_workspace_with_profile("debug")
}

pub fn build_workspace_with_profile(profile: &str) -> Result<(), String> {
    println!("Resolving workspace...");
    
    let workspace_root = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let workspace = ResolvedWorkspace::resolve(workspace_root)?;
    
    println!("Found {} packages", workspace.packages.len());
    println!("Build order: {:?}", workspace.build_order);
    println!("Profile: {}", profile);
    
    // Get target directory for this profile
    let target_dir = if profile == "release" {
        workspace.manifest.target.release.clone()
    } else {
        workspace.manifest.target.debug.clone()
    };
    
    // Build each package in order
    for package_name in &workspace.build_order {
        let package = workspace.get_package(package_name)
            .ok_or_else(|| format!("Package not found: {}", package_name))?;
        
        println!("\nBuilding {}...", package.name);
        build_package(package, &target_dir, profile)?;
    }
    
    println!("\nBuild complete!");
    Ok(())
}

fn build_package(package: &crate::resolver::ResolvedPackage, target_dir: &PathBuf, profile: &str) -> Result<(), String> {
    // Build each binary target
    for bin in &package.manifest.bins {
        // Path is relative to src_dir, which is relative to manifest location
        let manifest_dir = package.manifest_path.parent().unwrap_or(&package.path);
        let source_path = manifest_dir.join(&package.manifest.src_dir).join(&bin.path);
        
        if !source_path.exists() {
            eprintln!("Warning: Source file not found: {}", source_path.display());
            continue;
        }
        
        let output_path = target_dir.join(&bin.name);
        
        println!("  Compiling {} -> {}", bin.name, output_path.display());
        
        let mut cmd = Command::new("rustc");
        cmd.arg(&source_path)
           .arg("-o")
           .arg(&output_path)
           .arg("--edition")
           .arg(&package.manifest.edition);
        
        // Add optimization flags based on profile
        if profile == "release" {
            cmd.arg("-C").arg("opt-level=3");
        } else {
            cmd.arg("-C").arg("opt-level=0");
            cmd.arg("-C").arg("debuginfo=2");
        }
        
        // Add output directory for dependencies
        let deps_dir = target_dir.join("deps");
        std::fs::create_dir_all(&deps_dir)
            .map_err(|e| format!("Failed to create deps dir: {}", e))?;
        
        cmd.arg("--out-dir")
           .arg(&deps_dir);
        
        // Get the result
        let result = cmd.output()
            .map_err(|e| format!("Failed to invoke rustc: {}", e))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Compilation failed for {}:\n{}", bin.name, stderr));
        }
    }
    
    Ok(())
}
