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
        
        let (pkg_passed, pkg_failed, pkg_tests) = run_package_tests(package, filter)?;
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

fn run_package_tests(package: &crate::resolver::ResolvedPackage, filter: Option<&str>) -> Result<(usize, usize, usize), String> {
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
            let (lib_passed, lib_failed, lib_total) = run_inline_tests(package, lib, filter)?;
            passed += lib_passed;
            failed += lib_failed;
            total += lib_total;
        }
    }
    
    // 2. Run integration tests from tests/ directory
    let (int_passed, int_failed, int_total) = run_integration_tests(package, filter)?;
    passed += int_passed;
    failed += int_failed;
    total += int_total;
    
    Ok((passed, failed, total))
}

fn run_inline_tests(package: &crate::resolver::ResolvedPackage, lib: &crate::manifest::LibraryTarget, filter: Option<&str>) -> Result<(usize, usize, usize), String> {
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

fn run_integration_tests(package: &crate::resolver::ResolvedPackage, filter: Option<&str>) -> Result<(usize, usize, usize), String> {
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
        
        // Link against library if present
        if let Some(lib) = &package.manifest.lib {
            let lib_name = package.manifest.name.replace('-', "_");
            let lib_path = deps_dir.join(format!("lib{}.rlib", lib_name));
            if lib_path.exists() {
                cmd.arg("-L").arg(format!("{}={}", deps_dir.display(), deps_dir.display()));
                cmd.arg("--extern").arg(format!("{}={}", lib_name, lib_path.display()));
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
