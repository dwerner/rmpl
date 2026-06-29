//! Install command - installs binaries to ~/.rmpl/bin

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::resolver::ResolvedWorkspace;

pub fn install_workspace(profile: &str, force: bool) -> Result<(), String> {
    println!("Resolving workspace...");
    
    let workspace_root = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let workspace = ResolvedWorkspace::resolve(workspace_root)?;
    
    // Get install directory
    let install_dir = get_install_dir()?;
    
    println!("Found {} packages", workspace.packages.len());
    println!("Install directory: {}", install_dir.display());
    
    // Install each binary
    let mut installed = 0;
    for package_name in &workspace.build_order {
        let package = workspace.get_package(package_name)
            .ok_or_else(|| format!("Package not found: {}", package_name))?;
        
        // Only install binaries, not libraries
        if package.manifest.bins.is_empty() {
            continue;
        }
        
        println!("\nInstalling {}...", package.name);
        
        for bin in &package.manifest.bins {
            let target_dir = if profile == "release" {
                workspace.manifest.profiles.get("release")
                    .map(|p| p.target.clone())
                    .unwrap_or_else(|| PathBuf::from("target/release"))
            } else {
                workspace.manifest.profiles.get("debug")
                    .map(|p| p.target.clone())
                    .unwrap_or_else(|| PathBuf::from("target/debug"))
            };
            
            let bin_path = target_dir.join(&bin.name);
            
            if !bin_path.exists() {
                println!("  Warning: Binary not found: {}", bin_path.display());
                println!("  Run 'rmpl build {}' first", profile);
                continue;
            }
            
            let install_path = install_dir.join(&bin.name);
            
            if install_path.exists() && !force {
                println!("  Skipping {} (already exists, use --force to overwrite)", bin.name);
                continue;
            }
            
            copy_binary(&bin_path, &install_path)?;
            installed += 1;
            println!("  Installed {} -> {}", bin.name, install_path.display());
        }
    }
    
    println!("\nInstalled {} binary/ies to {}", installed, install_dir.display());
    
    // Add to PATH if not already present
    add_to_path(&install_dir, force)?;
    
    Ok(())
}

fn get_install_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;
    
    let install_dir = PathBuf::from(home).join(".rmpl").join("bin");
    
    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)
            .map_err(|e| format!("Failed to create install directory: {}", e))?;
    }
    
    Ok(install_dir)
}

fn copy_binary(src: &Path, dst: &Path) -> Result<(), String> {
    // On Unix, we can use rename/move if on same filesystem, otherwise copy
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        // Try to preserve executable permissions
        let metadata = fs::metadata(src)
            .map_err(|e| format!("Failed to read source metadata: {}", e))?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        
        fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy binary: {}", e))?;
        
        fs::set_permissions(dst, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    
    #[cfg(not(unix))]
    {
        fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy binary: {}", e))?;
    }
    
    Ok(())
}

fn add_to_path(install_dir: &Path, force: bool) -> Result<(), String> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;
    
    // Try .zshrc first, then .bashrc
    let shell_configs = vec![
        PathBuf::from(&home).join(".zshrc"),
        PathBuf::from(&home).join(".bashrc"),
    ];
    
    let path_line = format!("export PATH=\"{}:$PATH\"", install_dir.display());
    
    for config_path in &shell_configs {
        if !config_path.exists() {
            continue;
        }
        
        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read {}: {}", config_path.display(), e))?;
        
        // Check if path is already in config
        if content.contains(&install_dir.display().to_string()) {
            if force {
                println!("  PATH already in {} (use --force to update)", 
                    config_path.file_name().unwrap_or_default().to_string_lossy());
            }
            return Ok(());
        }
        
        // Append to config
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push('\n');
        new_content.push_str("# rmpl binaries\n");
        new_content.push_str(&path_line);
        new_content.push('\n');
        
        fs::write(config_path, &new_content)
            .map_err(|e| format!("Failed to write {}: {}", config_path.display(), e))?;
        
        println!("  Added to PATH in {}", 
            config_path.file_name().unwrap_or_default().to_string_lossy());
        return Ok(());
    }
    
    // No shell config found, just print instructions
    println!("  Add {} to your PATH manually in your shell config", 
        install_dir.display());
    
    Ok(())
}
