//! Manifest parser - parses rmpl.yaml and package.yaml without external dependencies

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceManifest {
    pub root: PathBuf,
    pub name: String,
    pub members: Vec<String>,
    pub dependencies: HashMap<String, Dependency>,
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Clone)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub description: Option<String>,
    pub src_dir: PathBuf,
    pub bins: Vec<BinaryTarget>,
    pub lib: Option<LibraryTarget>,
    pub tests: Vec<TestTarget>,
    pub benches: Vec<BenchTarget>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub target: PathBuf,
    pub opt_level: u32,
    pub debug: bool,
    pub lto: bool,
    pub strip: bool,
}

#[derive(Debug, Clone)]
pub struct BinaryTarget {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LibraryTarget {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestTarget {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BenchTarget {
    pub name: String,
    pub path: PathBuf,
}

impl WorkspaceManifest {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        
        Self::parse(&content, path.parent().unwrap().to_path_buf())
    }
    
    fn parse(content: &str, root: PathBuf) -> Result<Self, String> {
        let mut members = Vec::new();
        let mut dependencies = HashMap::new();
        let mut profiles = HashMap::new();
        let mut name = String::from("unnamed");
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_section = String::new();
        let mut in_members_list = false;
        let mut in_deps_block = false;
        let mut current_dep_name = String::new();
        let mut in_profile = false;
        let mut current_profile_name = String::new();
        
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }
            
            // Top-level sections
            if !line.starts_with(' ') && !line.starts_with('\t') {
                if trimmed == "workspace:" {
                    current_section = "workspace".to_string();
                } else if trimmed == "profiles:" {
                    current_section = "profiles".to_string();
                } else if trimmed.starts_with("package:") {
                    current_section = "package".to_string();
                }
                in_members_list = false;
                in_deps_block = false;
                in_profile = false;
                i += 1;
                continue;
            }
            
            match current_section.as_str() {
                "workspace" => {
                    if trimmed.starts_with("name:") {
                        name = extract_value(trimmed);
                    } else if trimmed.starts_with("members:") {
                        in_members_list = true;
                        in_deps_block = false;
                    } else if trimmed.starts_with("dependencies:") {
                        in_members_list = false;
                        in_deps_block = true;
                    } else if in_members_list && trimmed.starts_with("- ") {
                        let member = trimmed[2..].trim().trim_matches('"').to_string();
                        members.push(member);
                    } else if in_deps_block {
                        if trimmed.contains(':') {
                            let parts: Vec<&str> = trimmed.split(':').collect();
                            if parts.len() >= 2 {
                                current_dep_name = parts[0].trim().to_string();
                                let dep = Dependency {
                                    version: extract_value(parts[1].trim()),
                                    features: Vec::new(),
                                };
                                dependencies.insert(current_dep_name.clone(), dep);
                            }
                        }
                    }
                }
                "target" => {
                    if trimmed.starts_with("debug:") {
                        target.debug = PathBuf::from(extract_value(trimmed));
                    } else if trimmed.starts_with("release:") {
                        target.release = PathBuf::from(extract_value(trimmed));
                    }
                }
                "profiles" => {
                    if !trimmed.starts_with(' ') && trimmed.ends_with(':') {
                        current_profile_name = trimmed.trim_end_matches(':').to_string();
                        in_profile = true;
                    } else if in_profile {
                        let profile = profiles.entry(current_profile_name.clone())
                            .or_insert(Profile {
                                opt_level: 0,
                                debug: false,
                                lto: false,
                                strip: false,
                            });
                        
                        if trimmed.starts_with("opt_level:") {
                            profile.opt_level = extract_value(trimmed).parse().unwrap_or(0);
                        } else if trimmed.starts_with("debug:") {
                            profile.debug = extract_value(trimmed) == "true";
                        } else if trimmed.starts_with("lto:") {
                            profile.lto = extract_value(trimmed) == "true";
                        } else if trimmed.starts_with("strip:") {
                            profile.strip = extract_value(trimmed) == "true";
                        }
                    }
                }
                _ => {}
            }
            
            i += 1;
        }
        
        Ok(WorkspaceManifest {
            root,
            name,
            members,
            dependencies,
            profiles,
            target,
        })
    }
}

impl PackageManifest {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        
        Self::parse(&content)
    }
    
    fn parse(content: &str) -> Result<Self, String> {
        let mut package = PackageManifest {
            name: String::new(),
            version: String::from("0.1.0"),
            edition: String::from("2021"),
            description: None,
            src_dir: PathBuf::from("src"),
            bins: Vec::new(),
            lib: None,
            tests: Vec::new(),
            benches: Vec::new(),
            dependencies: HashMap::new(),
        };
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_section = String::new();
        let mut in_bin_list = false;
        let mut current_bin_name = String::new();
        let mut current_bin_path = String::new();
        
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }
            
            // Top-level sections
            if !line.starts_with(' ') && !line.starts_with('\t') {
                current_section = trimmed.trim_end_matches(':').to_string();
                in_bin_list = false;
                i += 1;
                continue;
            }
            
            match current_section.as_str() {
                "package" => {
                    if trimmed.starts_with("name:") {
                        package.name = extract_value(trimmed);
                    } else if trimmed.starts_with("version:") {
                        package.version = extract_value(trimmed);
                    } else if trimmed.starts_with("edition:") {
                        package.edition = extract_value(trimmed);
                    } else if trimmed.starts_with("description:") {
                        package.description = Some(extract_value(trimmed));
                    } else if trimmed.starts_with("src_dir:") {
                        package.src_dir = PathBuf::from(extract_value(trimmed));
                    }
                }
                "bin" => {
                    if trimmed.starts_with("- name:") {
                        if !current_bin_name.is_empty() && !current_bin_path.is_empty() {
                            package.bins.push(BinaryTarget {
                                name: current_bin_name.clone(),
                                path: PathBuf::from(current_bin_path.clone()),
                            });
                        }
                        current_bin_name = extract_value(trimmed);
                        current_bin_path = String::new();
                    } else if trimmed.starts_with("path:") {
                        // Path is relative to src_dir
                        current_bin_path = extract_value(trimmed);
                    }
                }
                "dependencies" => {
                    if trimmed.contains(':') {
                        let parts: Vec<&str> = trimmed.split(':').collect();
                        if parts.len() >= 2 {
                            let dep_name = parts[0].trim().to_string();
                            let dep_spec = parts[1].trim().trim_matches('"').to_string();
                            package.dependencies.insert(dep_name, dep_spec);
                        }
                    }
                }
                _ => {}
            }
            
            i += 1;
        }
        
        // Add last binary if any
        if !current_bin_name.is_empty() && !current_bin_path.is_empty() {
            package.bins.push(BinaryTarget {
                name: current_bin_name,
                path: PathBuf::from(current_bin_path),
            });
        }
        
        // Default to src/main.rs if no bins specified
        if package.bins.is_empty() {
            package.bins.push(BinaryTarget {
                name: package.name.clone(),
                path: PathBuf::from("src/main.rs"),
            });
        }
        
        Ok(package)
    }
}

fn extract_value(line: &str) -> String {
    line.split(':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_value() {
        assert_eq!(extract_value("name: \"test\""), "test");
        assert_eq!(extract_value("version: 1.0.0"), "1.0.0");
    }
}
