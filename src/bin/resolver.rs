//! Workspace resolver - discovers packages and builds dependency graph

use crate::manifest::{WorkspaceManifest, PackageManifest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ResolvedPackage {
    pub name: String,
    pub manifest: PackageManifest,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub struct ResolvedWorkspace {
    pub manifest: WorkspaceManifest,
    pub packages: HashMap<String, ResolvedPackage>,
    pub build_order: Vec<String>,
}

impl ResolvedWorkspace {
    pub fn resolve(workspace_root: PathBuf) -> Result<Self, String> {
        let manifest = WorkspaceManifest::load(workspace_root.join("workspace.yaml"))?;
        
        let mut packages = HashMap::new();
        
        // Resolve each member package
        for member_path in &manifest.members {
            let package_path = manifest.root.join(member_path);
            let package_manifest_path = package_path.join("package.yaml");
            
            if package_manifest_path.exists() {
                let pkg_manifest = PackageManifest::load(package_manifest_path.clone())?;
                
                // Resolve dependencies - only track local package dependencies
                let mut dependencies = Vec::new();
                for (dep_name, dep_spec) in &pkg_manifest.dependencies {
                    if dep_spec == "workspace" {
                        // Check if this is a workspace dependency that points to a local package
                        if let Some(dep) = manifest.dependencies.get(dep_name) {
                            dependencies.push(format!("{} {}", dep_name, dep.version));
                        }
                    } else {
                        // External dependency - will be handled by vendoring
                        dependencies.push(format!("{} {}", dep_name, dep_spec));
                    }
                }
                
                let resolved = ResolvedPackage {
                    name: pkg_manifest.name.clone(),
                    manifest: pkg_manifest,
                    path: package_path,
                    manifest_path: package_manifest_path,
                    dependencies,
                };
                
                packages.insert(resolved.name.clone(), resolved);
            }
        }
        
        // Compute build order (topological sort)
        let build_order = Self::compute_build_order(&packages)?;
        
        Ok(ResolvedWorkspace {
            manifest,
            packages,
            build_order,
        })
    }
    
    fn compute_build_order(packages: &HashMap<String, ResolvedPackage>) -> Result<Vec<String>, String> {
        let mut visited = HashMap::new();
        let mut order = Vec::new();
        
        fn visit(
            name: &str,
            packages: &HashMap<String, ResolvedPackage>,
            visited: &mut HashMap<String, bool>,
            order: &mut Vec<String>,
        ) -> Result<(), String> {
            if visited.contains_key(name) {
                if visited.get(name) == Some(&true) {
                    return Ok(()); // Already processed
                }
                // Cycle detected
                return Err(format!("Circular dependency detected: {}", name));
            }
            
            visited.insert(name.to_string(), false); // Mark as being processed
            
            if let Some(pkg) = packages.get(name) {
                // Visit dependencies first (including proc macro dependencies)
                for dep in &pkg.dependencies {
                    let dep_name = dep.split(' ').next().unwrap_or(dep);
                    // Only visit if it's a local package we're building
                    if packages.contains_key(dep_name) {
                        visit(dep_name, packages, visited, order)?;
                    }
                }
            }
            
            visited.insert(name.to_string(), true); // Mark as processed
            order.push(name.to_string());
            
            Ok(())
        }
        
        let mut names: Vec<&String> = packages.keys().collect();
        names.sort();
        for name in names {
            if !visited.contains_key(name) {
                visit(name, packages, &mut visited, &mut order)?;
            }
        }
        
        Ok(order)
    }
    
    pub fn get_package(&self, name: &str) -> Option<&ResolvedPackage> {
        self.packages.get(name)
    }
}
