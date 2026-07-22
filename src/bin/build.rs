//! Build orchestrator - invokes rustc to compile packages

use crate::resolver::ResolvedWorkspace;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::env::consts::DLL_EXTENSION;

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
    let target_dir = workspace.manifest.profiles.get(profile)
        .map(|p| p.target.clone())
        .unwrap_or_else(|| PathBuf::from(format!("target/{}", profile)));
    
    // Build each package in order
    for package_name in &workspace.build_order {
        let package = workspace.get_package(package_name)
            .ok_or_else(|| format!("Package not found: {}", package_name))?;
        
        println!("\nBuilding {}...", package.name);
        build_package(package, &target_dir, profile, &workspace)?;
    }
    
    println!("\nBuild complete!");
    Ok(())
}

fn build_package(
    package: &crate::resolver::ResolvedPackage, 
    target_dir: &PathBuf, 
    profile: &str,
    workspace: &crate::resolver::ResolvedWorkspace,
) -> Result<(), String> {
    let lib_name = package.manifest.name.replace('-', "_");
    
    // Collect all library dependencies for this package (including transitive)
    let mut dep_libs = Vec::new();
    let mut visited = std::collections::HashSet::new();
    
    fn collect_deps(
        dep_name: &str, 
        workspace: &crate::resolver::ResolvedWorkspace, 
        dep_libs: &mut Vec<(String, String)>,
        visited: &mut std::collections::HashSet<String>
    ) {
        if visited.contains(dep_name) {
            return;
        }
        visited.insert(dep_name.to_string());
        
        if let Some(dep_pkg) = workspace.packages.get(dep_name) {
            if dep_pkg.manifest.lib.is_some() {
                let dep_lib_name = dep_name.replace('-', "_");
                dep_libs.push((dep_lib_name, dep_name.to_string()));
            }
            // Recursively collect transitive dependencies
            for dep in &dep_pkg.dependencies {
                let sub_dep = dep.split(' ').next().unwrap_or(dep);
                collect_deps(sub_dep, workspace, dep_libs, visited);
            }
        }
    }
    
    for dep in &package.dependencies {
        let dep_name = dep.split(' ').next().unwrap_or(dep);
        collect_deps(dep_name, workspace, &mut dep_libs, &mut visited);
    }
    
    // Remove duplicates while preserving order
    let mut seen = std::collections::HashSet::new();
    dep_libs.retain(|(name, _)| seen.insert(name.clone()));
    
    // Build library if present
    if let Some(lib) = &package.manifest.lib {
        let manifest_dir = package.manifest_path.parent().unwrap_or(&package.path);
        let source_path = manifest_dir.join(&lib.path);
        
        if source_path.exists() {
            // Proc macros produce dynamic libraries, regular libs produce rlib
            let (crate_type, output_ext) = if package.manifest.proc_macro {
                ("proc-macro", get_dylib_extension())
            } else {
                ("lib", "rlib")
            };
            
            let output_filename = if package.manifest.proc_macro {
                format!("lib{}.{}", lib_name, output_ext)
            } else {
                format!("lib{}.rlib", lib_name)
            };
            
            let output_path = target_dir.join("deps").join(&output_filename);
            
            println!("  Compiling {} {} -> {}", 
                if package.manifest.proc_macro { "proc-macro" } else { "lib" }, 
                lib_name, 
                output_path.display());
            
            let mut cmd = Command::new("rustc");
            cmd.arg("--crate-type").arg(crate_type)
               .arg("--crate-name").arg(&lib_name)
               .arg(&source_path)
               .arg("--edition")
               .arg(&package.manifest.edition);
            
            // Add library dependencies
            let deps_dir = target_dir.join("deps");
            for (dep_lib_name, _) in &dep_libs {
                cmd.arg("-L").arg(format!("dependency={}", deps_dir.display()));
                let dep_rlib = deps_dir.join(format!("lib{}.rlib", dep_lib_name));
                if dep_rlib.exists() {
                    let extern_arg = format!("{}={}", dep_lib_name, dep_rlib.display());
                    cmd.arg("--extern").arg(&extern_arg);
                }
            }
            
            // Proc macros use --out-dir only, regular libs use -o
            if package.manifest.proc_macro {
                let deps_dir = target_dir.join("deps");
                std::fs::create_dir_all(&deps_dir)
                    .map_err(|e| format!("Failed to create deps dir: {}", e))?;
                cmd.arg("--out-dir").arg(&deps_dir);
            } else {
                cmd.arg("-o").arg(&output_path);
            }
            
            if profile == "release" {
                cmd.arg("-C").arg("opt-level=3");
            } else {
                cmd.arg("-C").arg("opt-level=0");
                cmd.arg("-C").arg("debuginfo=2");
            }
            
            // Create deps dir (needed for both proc-macro and regular lib)
            let deps_dir = target_dir.join("deps");
            std::fs::create_dir_all(&deps_dir)
                .map_err(|e| format!("Failed to create deps dir: {}", e))?;
            
            let result = cmd.output()
                .map_err(|e| format!("Failed to invoke rustc: {}", e))?;
            
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return Err(format!("Compilation failed for {} {}:\n{}", 
                    if package.manifest.proc_macro { "proc-macro" } else { "lib" },
                    lib_name, stderr));
            }
        }
    }
    
    // Build each binary target
    for bin in &package.manifest.bins {
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
        
        // Link against library if present in same package
        if package.manifest.lib.is_some() {
            let deps_dir = target_dir.join("deps");
            cmd.arg("-L").arg(format!("{}={}", deps_dir.display(), deps_dir.display()));
            cmd.arg("--extern").arg(format!("{}={}", lib_name, deps_dir.join(format!("lib{}.rlib", lib_name)).display()));
        }
        
        // Link against workspace dependencies
        let deps_dir = target_dir.join("deps");
        for (dep_lib_name, _) in &dep_libs {
            cmd.arg("-L").arg(format!("{}={}", deps_dir.display(), deps_dir.display()));
            let dep_rlib = deps_dir.join(format!("lib{}.rlib", dep_lib_name));
            if dep_rlib.exists() {
                cmd.arg("--extern").arg(format!("{}={}", dep_lib_name, dep_rlib.display()));
            }
        }
        
        if profile == "release" {
            cmd.arg("-C").arg("opt-level=3");
        } else {
            cmd.arg("-C").arg("opt-level=0");
            cmd.arg("-C").arg("debuginfo=2");
        }
        
        let deps_dir = target_dir.join("deps");
        std::fs::create_dir_all(&deps_dir)
            .map_err(|e| format!("Failed to create deps dir: {}", e))?;
        
        cmd.arg("--out-dir")
           .arg(&deps_dir);
        
        let result = cmd.output()
            .map_err(|e| format!("Failed to invoke rustc: {}", e))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Compilation failed for {}:\n{}", bin.name, stderr));
        }
    }
    
    Ok(())
}

fn get_dylib_extension() -> &'static str {
    std::env::consts::DLL_EXTENSION
}
