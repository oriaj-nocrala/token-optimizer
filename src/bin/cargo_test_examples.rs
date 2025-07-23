/*! Cargo Analyzer Test Examples
 * Demonstrates CargoAnalyzer parsing different types of Cargo.toml configurations
 */

use anyhow::Result;
use token_optimizer::analyzers::rust_analyzer::CargoAnalyzer;
use token_optimizer::types::{CargoDependencySource, CargoTargetType};

fn main() -> Result<()> {
    println!("🧪 CargoAnalyzer Test Examples");
    println!("==============================\n");
    
    // Test 1: Simple package with basic dependencies
    test_simple_package()?;
    
    // Test 2: Complex package with various dependency types
    test_complex_dependencies()?;
    
    // Test 3: Package with features
    test_features()?;
    
    // Test 4: Package with multiple targets
    test_targets()?;
    
    // Test 5: Workspace configuration
    test_workspace()?;
    
    println!("✅ All CargoAnalyzer tests completed successfully!");
    
    Ok(())
}

fn test_simple_package() -> Result<()> {
    println!("📋 Test 1: Simple Package");
    println!("--------------------------");
    
    let toml_content = r#"
[package]
name = "simple-app"
version = "1.0.0"
edition = "2021"
description = "A simple application"
license = "MIT"

[dependencies]
serde = "1.0"
tokio = "1.0"
anyhow = "1.0"
    "#;
    
    let cargo_info = CargoAnalyzer::analyze_cargo_toml(toml_content)?;
    
    println!("Package: {} v{}", cargo_info.package_name, cargo_info.version);
    println!("Dependencies: {}", cargo_info.dependencies.len());
    
    // Verify we found the expected dependencies
    let dep_names: Vec<_> = cargo_info.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"serde".to_string()));
    assert!(dep_names.contains(&&"tokio".to_string()));
    assert!(dep_names.contains(&&"anyhow".to_string()));
    
    println!("✅ Simple package test passed\n");
    Ok(())
}

fn test_complex_dependencies() -> Result<()> {
    println!("📋 Test 2: Complex Dependencies");
    println!("--------------------------------");
    
    let toml_content = r#"
[package]
name = "complex-app"
version = "0.2.0"
edition = "2021"

[dependencies]
# Simple version
serde = "1.0"

# With features
tokio = { version = "1.0", features = ["full", "macros"] }

# Path dependency
local-utils = { path = "../utils" }

# Git dependency
async-trait = { git = "https://github.com/dtolnay/async-trait.git", branch = "master" }

# Optional dependency
clap = { version = "4.0", optional = true }

# Without default features
reqwest = { version = "0.11", default-features = false, features = ["json"] }

[dev-dependencies]
tempfile = "3.0"
criterion = { version = "0.4", features = ["html_reports"] }

[build-dependencies]
cc = "1.0"
    "#;
    
    let cargo_info = CargoAnalyzer::analyze_cargo_toml(toml_content)?;
    
    println!("Package: {} v{}", cargo_info.package_name, cargo_info.version);
    println!("Regular dependencies: {}", cargo_info.dependencies.len());
    println!("Dev dependencies: {}", cargo_info.dev_dependencies.len());
    println!("Build dependencies: {}", cargo_info.build_dependencies.len());
    
    // Test tokio with features
    let tokio = cargo_info.dependencies.iter().find(|d| d.name == "tokio").unwrap();
    assert_eq!(tokio.features, vec!["full", "macros"]);
    println!("✓ Tokio features: {:?}", tokio.features);
    
    // Test path dependency
    let local_utils = cargo_info.dependencies.iter().find(|d| d.name == "local-utils").unwrap();
    if let CargoDependencySource::Path { path } = &local_utils.source {
        assert_eq!(path, "../utils");
        println!("✓ Path dependency: {}", path);
    }
    
    // Test git dependency
    let async_trait = cargo_info.dependencies.iter().find(|d| d.name == "async-trait").unwrap();
    if let CargoDependencySource::Git { url, branch, .. } = &async_trait.source {
        assert!(url.contains("async-trait.git"));
        assert_eq!(branch, &Some("master".to_string()));
        println!("✓ Git dependency: {} (branch: {:?})", url, branch);
    }
    
    // Test optional dependency
    let clap = cargo_info.dependencies.iter().find(|d| d.name == "clap").unwrap();
    assert!(clap.optional);
    println!("✓ Optional dependency: clap");
    
    // Test without default features
    let reqwest = cargo_info.dependencies.iter().find(|d| d.name == "reqwest").unwrap();
    assert!(!reqwest.default_features);
    assert_eq!(reqwest.features, vec!["json"]);
    println!("✓ No default features + custom features: reqwest");
    
    println!("✅ Complex dependencies test passed\n");
    Ok(())
}

fn test_features() -> Result<()> {
    println!("📋 Test 3: Features");
    println!("--------------------");
    
    let toml_content = r#"
[package]
name = "feature-app"
version = "0.1.0"
edition = "2021"

[features]
default = ["std", "serde"]
std = []
serde = ["dep:serde"]
async = ["tokio"]
full = ["std", "serde", "async"]

[dependencies]
serde = { version = "1.0", optional = true }
tokio = { version = "1.0", optional = true }
    "#;
    
    let cargo_info = CargoAnalyzer::analyze_cargo_toml(toml_content)?;
    
    println!("Package: {} v{}", cargo_info.package_name, cargo_info.version);
    println!("Features: {}", cargo_info.features.len());
    
    // Test default feature
    let default = cargo_info.features.iter().find(|f| f.name == "default").unwrap();
    assert_eq!(default.dependencies, vec!["std", "serde"]);
    println!("✓ Default feature: {:?}", default.dependencies);
    
    // Test full feature
    let full = cargo_info.features.iter().find(|f| f.name == "full").unwrap();
    assert_eq!(full.dependencies, vec!["std", "serde", "async"]);
    println!("✓ Full feature: {:?}", full.dependencies);
    
    println!("✅ Features test passed\n");
    Ok(())
}

fn test_targets() -> Result<()> {
    println!("📋 Test 4: Build Targets");
    println!("-------------------------");
    
    let toml_content = r#"
[package]
name = "multi-target-app"
version = "0.1.0"
edition = "2021"

[lib]
name = "mylib"
path = "src/lib.rs"

[[bin]]
name = "cli"
path = "src/bin/cli.rs"

[[bin]]
name = "server"
path = "src/bin/server.rs"

[[example]]
name = "demo"
path = "examples/demo.rs"

[[test]]
name = "integration"
path = "tests/integration.rs"

[[bench]]
name = "performance"
path = "benches/performance.rs"
required-features = ["benchmark"]
    "#;
    
    let cargo_info = CargoAnalyzer::analyze_cargo_toml(toml_content)?;
    
    println!("Package: {} v{}", cargo_info.package_name, cargo_info.version);
    println!("Targets: {}", cargo_info.targets.len());
    
    // Test library target
    let lib = cargo_info.targets.iter().find(|t| t.name == "mylib").unwrap();
    assert!(matches!(lib.target_type, CargoTargetType::Library));
    assert_eq!(lib.path, "src/lib.rs");
    println!("✓ Library: {} -> {}", lib.name, lib.path);
    
    // Test binary targets
    let binaries: Vec<_> = cargo_info.targets.iter()
        .filter(|t| matches!(t.target_type, CargoTargetType::Binary))
        .collect();
    assert_eq!(binaries.len(), 2);
    println!("✓ Found {} binary targets", binaries.len());
    
    // Test benchmark with required features
    let bench = cargo_info.targets.iter().find(|t| t.name == "performance").unwrap();
    assert!(matches!(bench.target_type, CargoTargetType::Benchmark));
    assert_eq!(bench.required_features, vec!["benchmark"]);
    println!("✓ Benchmark with required features: {:?}", bench.required_features);
    
    println!("✅ Build targets test passed\n");
    Ok(())
}

fn test_workspace() -> Result<()> {
    println!("📋 Test 5: Workspace Configuration");
    println!("-----------------------------------");
    
    let toml_content = r#"
[workspace]
members = [
    "crate-a",
    "crate-b", 
    "tools/*"
]
exclude = [
    "old-crate",
    "experimental/*"
]
default-members = ["crate-a"]

[package]
name = "workspace-root"
version = "0.1.0"
edition = "2021"
    "#;
    
    let cargo_info = CargoAnalyzer::analyze_cargo_toml(toml_content)?;
    
    println!("Package: {} v{}", cargo_info.package_name, cargo_info.version);
    
    assert!(cargo_info.workspace.is_some());
    let workspace = cargo_info.workspace.unwrap();
    
    assert_eq!(workspace.members, vec!["crate-a", "crate-b", "tools/*"]);
    assert_eq!(workspace.exclude, vec!["old-crate", "experimental/*"]);
    assert_eq!(workspace.default_members, vec!["crate-a"]);
    
    println!("✓ Workspace members: {:?}", workspace.members);
    println!("✓ Workspace exclude: {:?}", workspace.exclude);
    println!("✓ Default members: {:?}", workspace.default_members);
    
    println!("✅ Workspace test passed\n");
    Ok(())
}