//! Test runner - compiles and runs tests

use crate::resolver::ResolvedWorkspace;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_tests(workspace_root: PathBuf, filter: Option<&str>) -> Result<(), String> {
    println!("Resolving workspace...");
    
    let workspace = ResolvedWorkspace::resolve(workspace_root)?;
    
    println!("Found {} packages", workspace.packages.len());
    
    let mut total_tests = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Run tests for each package
    for package_name in &workspace.build_order {
        let package = workspace.get_package(package_name)
            .ok_or_else(|| format!("Package not found: {}", package_name))?;
        
        println!("\nTesting {}...", package.name);
        
        let (pkg_passed, pkg_failed, pkg_tests) = run_package_tests(package, &workspace, filter)?;
        passed += pkg_passed;
        failed += pkg_failed;
        total_tests += pkg_tests;
    }
    
    println!("\nTest summary: {}/{} passed, {} failed", passed, total_tests, failed);
    
    if failed > 0 {
        Err(format!("{} tests failed", failed))
    } else {
        Ok(())
    }
}

fn run_package_tests(
    package: &crate::resolver::ResolvedPackage, 
    workspace: &crate::resolver::ResolvedWorkspace,
    filter: Option<&str>
) -> Result<(usize, usize, usize), String> {
    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;
    
    let target_dir = PathBuf::from("target/debug");
    let deps_dir = target_dir.join("deps");
    std::fs::create_dir_all(&deps_dir)
        .map_err(|e| format!("Failed to create deps dir: {}", e))?;
    
    // 1. Run inline tests from lib.rs if present
    if let Some(lib) = &package.manifest.lib {
        if !package.manifest.proc_macro {
            let (lib_passed, lib_failed, lib_total) = run_inline_tests(package, lib, workspace, filter)?;
            passed += lib_passed;
            failed += lib_failed;
            total += lib_total;
        }
    }
    
    // 2. Run integration tests from tests/ directory
    let (int_passed, int_failed, int_total) = run_integration_tests(package, workspace, filter)?;
    passed += int_passed;
    failed += int_failed;
    total += int_total;
    
    Ok((passed, failed, total))
}

fn run_inline_tests(
    package: &crate::resolver::ResolvedPackage, 
    lib: &crate::manifest::LibraryTarget, 
    workspace: &crate::resolver::ResolvedWorkspace,
    filter: Option<&str>
) -> Result<(usize, usize, usize), String> {
    let manifest_dir = package.manifest_path.parent().unwrap_or(&package.path);
    let source_path = manifest_dir.join(&lib.path);
    
    if !source_path.exists() {
        return Ok((0, 0, 0));
    }
    
    let lib_name = package.manifest.name.replace('-', "_");
    let test_binary = format!("{}_tests", lib_name);
    let output_path = PathBuf::from("target/debug/deps").join(&test_binary);
    
    println!("  Compiling inline tests...");
    
    let mut cmd = Command::new("rustc");
    cmd.arg("--test")
       .arg(&source_path)
       .arg("-o")
       .arg(&output_path)
       .arg("--edition")
       .arg(&package.manifest.edition);
    
    // Create deps dir
    let deps_dir = PathBuf::from("target/debug/deps");
    std::fs::create_dir_all(&deps_dir)
        .map_err(|e| format!("Failed to create deps dir: {}", e))?;
    
    cmd.arg("--out-dir").arg(&deps_dir);
    
    // Add library dependencies
    let mut dep_libs = Vec::new();
    let mut _visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    
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
            for dep in &dep_pkg.dependencies {
                let sub_dep = dep.split(' ').next().unwrap_or(dep);
                collect_deps(sub_dep, workspace, dep_libs, visited);
            }
        }
    }
    
    for dep in &package.dependencies {
        let dep_name = dep.split(' ').next().unwrap_or(dep);
        collect_deps(dep_name, workspace, &mut dep_libs, &mut _visited);
    }
    
    let mut seen = std::collections::HashSet::new();
    dep_libs.retain(|(name, _)| seen.insert(name.clone()));
    
    for (dep_lib_name, _) in &dep_libs {
        cmd.arg("-L").arg(format!("dependency={}", deps_dir.display()));
        let dep_rlib = deps_dir.join(format!("lib{}.rlib", dep_lib_name));
        if dep_rlib.exists() {
            cmd.arg("--extern").arg(format!("{}={}", dep_lib_name, dep_rlib.display()));
        }
    }
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to invoke rustc: {}", e))?;
    
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Test compilation failed for {}:\n{}", package.name, stderr));
    }
    
    // Run the test binary
    println!("  Running inline tests...");
    let mut run_cmd = Command::new(&output_path);
    
    if let Some(f) = filter {
        run_cmd.arg(f);
    }
    run_cmd.arg("--format").arg("pretty");
    
    let run_result = run_cmd.output()
        .map_err(|e| format!("Failed to run tests: {}", e))?;
    
    let output = String::from_utf8_lossy(&run_result.stdout);
    let errors = String::from_utf8_lossy(&run_result.stderr);
    
    // Parse test output to count passed/failed
    let (pkg_passed, pkg_failed) = parse_test_output(&output);
    
    if !run_result.status.success() {
        println!("  Test failures:");
        print!("{}", output);
        if !errors.is_empty() {
            print!("{}", errors);
        }
    }
    
    Ok((pkg_passed, pkg_failed, pkg_passed + pkg_failed))
}

fn run_integration_tests(
    package: &crate::resolver::ResolvedPackage, 
    workspace: &crate::resolver::ResolvedWorkspace,
    filter: Option<&str>
) -> Result<(usize, usize, usize), String> {
    let manifest_dir = package.manifest_path.parent().unwrap_or(&package.path);
    let tests_dir = manifest_dir.join("tests");
    
    if !tests_dir.exists() {
        return Ok((0, 0, 0));
    }
    
    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;
    
    // Find all .rs files in tests/
    let entries = fs::read_dir(&tests_dir)
        .map_err(|e| format!("Failed to read tests directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if !path.is_file() || path.extension().map_or(false, |e| e != "rs") {
            continue;
        }
        
        let test_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        println!("  Compiling test {}...", test_name);
        
        let output_path = PathBuf::from("target/debug/deps").join(format!("{}_test", test_name));
        
        let mut cmd = Command::new("rustc");
        cmd.arg("--test")
           .arg(&path)
           .arg("-o")
           .arg(&output_path)
           .arg("--edition")
           .arg(&package.manifest.edition);
        
        let deps_dir = PathBuf::from("target/debug/deps");
        std::fs::create_dir_all(&deps_dir)
            .map_err(|e| format!("Failed to create deps dir: {}", e))?;
        
        cmd.arg("--out-dir").arg(&deps_dir);
        
        // Add library dependencies
        let mut dep_libs = Vec::new();
        let mut _visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for dep in &package.dependencies {
            let dep_name = dep.split(' ').next().unwrap_or(dep);
            if let Some(dep_pkg) = workspace.packages.get(dep_name) {
                if dep_pkg.manifest.lib.is_some() {
                    let dep_lib_name = dep_name.replace('-', "_");
                    dep_libs.push((dep_lib_name, dep_name.to_string()));
                }
                for sub_dep in &dep_pkg.dependencies {
                    let sub_name = sub_dep.split(' ').next().unwrap_or(sub_dep);
                    if let Some(sub_pkg) = workspace.packages.get(sub_name) {
                        if sub_pkg.manifest.lib.is_some() {
                            let sub_lib_name = sub_name.replace('-', "_");
                            dep_libs.push((sub_lib_name, sub_name.to_string()));
                        }
                    }
                }
            }
        }
        
        let mut seen = std::collections::HashSet::new();
        dep_libs.retain(|(name, _)| seen.insert(name.clone()));
        
        for (dep_lib_name, _) in &dep_libs {
            cmd.arg("-L").arg(format!("dependency={}", deps_dir.display()));
            let dep_rlib = deps_dir.join(format!("lib{}.rlib", dep_lib_name));
            if dep_rlib.exists() {
                cmd.arg("--extern").arg(format!("{}={}", dep_lib_name, dep_rlib.display()));
            }
        }
        
        let result = cmd.output()
            .map_err(|e| format!("Failed to compile test {}: {}", test_name, e))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Test compilation failed for {}:\n{}", test_name, stderr));
        }
        
        // Run the test
        println!("  Running test {}...", test_name);
        let mut run_cmd = Command::new(&output_path);
        
        if let Some(f) = filter {
            run_cmd.arg(f);
        }
        run_cmd.arg("--format").arg("pretty");
        
        let run_result = run_cmd.output()
            .map_err(|e| format!("Failed to run test {}: {}", test_name, e))?;
        
        let output = String::from_utf8_lossy(&run_result.stdout);
        
        let (test_passed, test_failed) = parse_test_output(&output);
        passed += test_passed;
        failed += test_failed;
        total += test_passed + test_failed;
        
        if !run_result.status.success() {
            println!("  Test {} failed:", test_name);
            print!("{}", output);
        } else {
            println!("  Test {} passed", test_name);
        }
    }
    
    Ok((passed, failed, total))
}

fn parse_test_output(output: &str) -> (usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    
    for line in output.lines() {
        if line.contains("test result: ok") {
            // Parse "test result: ok. X passed; Y failed; Z ignored"
            if let Some(passed_str) = line.split("passed").next().and_then(|s| s.rsplit(' ').next()) {
                passed = passed_str.trim().parse().unwrap_or(0);
            }
            if let Some(failed_str) = line.split("failed").next().and_then(|s| s.rsplit(' ').next()) {
                failed = failed_str.trim().parse().unwrap_or(0);
            }
        }
    }
    
    // If we couldn't parse, check for individual test results
    if passed == 0 && failed == 0 {
        for line in output.lines() {
            if line.contains("... ok") {
                passed += 1;
            } else if line.contains("... FAILED") {
                failed += 1;
            }
        }
    }
    
    (passed, failed)
}
