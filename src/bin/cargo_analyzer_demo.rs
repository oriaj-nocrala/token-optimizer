/*! Cargo Analyzer Demo
 * A standalone test program that demonstrates the CargoAnalyzer functionality
 * by analyzing our own Cargo.toml file and displaying the results.
 */

use std::fs;
use anyhow::Result;
use token_optimizer::analyzers::rust_analyzer::CargoAnalyzer;
use token_optimizer::types::{CargoDependencySource, CargoTargetType};

fn main() -> Result<()> {
    println!("🔍 Cargo Analyzer Demo");
    println!("====================\n");
    
    // Read our own Cargo.toml file
    let cargo_toml_path = "Cargo.toml";
    println!("📄 Reading Cargo.toml from: {}", cargo_toml_path);
    
    let content = fs::read_to_string(cargo_toml_path)
        .map_err(|e| anyhow::anyhow!("Failed to read Cargo.toml: {}", e))?;
    
    println!("✅ Successfully read {} bytes from Cargo.toml\n", content.len());
    
    // Analyze the Cargo.toml content
    println!("🔬 Analyzing Cargo.toml with CargoAnalyzer...");
    let cargo_info = CargoAnalyzer::analyze_cargo_toml(&content)?;
    println!("✅ Analysis completed successfully!\n");
    
    // Display package information
    println!("📦 PACKAGE INFORMATION");
    println!("======================");
    println!("Name:     {}", cargo_info.package_name);
    println!("Version:  {}", cargo_info.version);
    println!("Edition:  {}", cargo_info.edition);
    println!();
    
    // Display regular dependencies
    println!("📚 REGULAR DEPENDENCIES ({} found)", cargo_info.dependencies.len());
    println!("===============================");
    for dep in &cargo_info.dependencies {
        print!("• {}", dep.name);
        
        if let Some(version) = &dep.version {
            print!(" = \"{}\"", version);
        }
        
        match &dep.source {
            CargoDependencySource::CratesIo => {
                // Default source, no extra info needed
            }
            CargoDependencySource::Git { url, branch, tag, rev } => {
                print!(" [Git: {}]", url);
                if let Some(branch) = branch {
                    print!(" (branch: {})", branch);
                }
                if let Some(tag) = tag {
                    print!(" (tag: {})", tag);
                }
                if let Some(rev) = rev {
                    print!(" (rev: {})", rev);
                }
            }
            CargoDependencySource::Path { path } => {
                print!(" [Path: {}]", path);
            }
        }
        
        if !dep.features.is_empty() {
            print!(" (features: [{}])", dep.features.join(", "));
        }
        
        if dep.optional {
            print!(" [OPTIONAL]");
        }
        
        if !dep.default_features {
            print!(" [NO DEFAULT FEATURES]");
        }
        
        println!();
    }
    println!();
    
    // Display dev dependencies
    if !cargo_info.dev_dependencies.is_empty() {
        println!("🧪 DEV DEPENDENCIES ({} found)", cargo_info.dev_dependencies.len());
        println!("==============================");
        for dep in &cargo_info.dev_dependencies {
            print!("• {}", dep.name);
            if let Some(version) = &dep.version {
                print!(" = \"{}\"", version);
            }
            if !dep.features.is_empty() {
                print!(" (features: [{}])", dep.features.join(", "));
            }
            println!();
        }
        println!();
    }
    
    // Display build dependencies
    if !cargo_info.build_dependencies.is_empty() {
        println!("🔨 BUILD DEPENDENCIES ({} found)", cargo_info.build_dependencies.len());
        println!("=================================");
        for dep in &cargo_info.build_dependencies {
            print!("• {}", dep.name);
            if let Some(version) = &dep.version {
                print!(" = \"{}\"", version);
            }
            if !dep.features.is_empty() {
                print!(" (features: [{}])", dep.features.join(", "));
            }
            println!();
        }
        println!();
    }
    
    // Display features
    if !cargo_info.features.is_empty() {
        println!("🎯 FEATURES ({} found)", cargo_info.features.len());
        println!("===================");
        for feature in &cargo_info.features {
            print!("• {}", feature.name);
            if feature.is_default {
                print!(" [DEFAULT]");
            }
            if !feature.dependencies.is_empty() {
                print!(" -> [{}]", feature.dependencies.join(", "));
            }
            println!();
        }
        println!();
    }
    
    // Display targets
    if !cargo_info.targets.is_empty() {
        println!("🎯 BUILD TARGETS ({} found)", cargo_info.targets.len());
        println!("========================");
        for target in &cargo_info.targets {
            let target_type_str = match target.target_type {
                CargoTargetType::Library => "Library",
                CargoTargetType::Binary => "Binary",
                CargoTargetType::Example => "Example",
                CargoTargetType::Test => "Test",
                CargoTargetType::Benchmark => "Benchmark",
            };
            
            println!("• {} [{}] -> {}", target.name, target_type_str, target.path);
            
            if !target.required_features.is_empty() {
                println!("  Required features: [{}]", target.required_features.join(", "));
            }
        }
        println!();
    }
    
    // Display workspace information
    if let Some(workspace) = &cargo_info.workspace {
        println!("🏢 WORKSPACE CONFIGURATION");
        println!("==========================");
        
        if !workspace.members.is_empty() {
            println!("Members: [{}]", workspace.members.join(", "));
        }
        
        if !workspace.exclude.is_empty() {
            println!("Exclude: [{}]", workspace.exclude.join(", "));
        }
        
        if !workspace.default_members.is_empty() {
            println!("Default members: [{}]", workspace.default_members.join(", "));
        }
        println!();
    }
    
    // Summary statistics
    println!("📊 ANALYSIS SUMMARY");
    println!("==================");
    println!("• Package: {} v{} (Rust {})", 
             cargo_info.package_name, cargo_info.version, cargo_info.edition);
    println!("• Dependencies: {} regular, {} dev, {} build", 
             cargo_info.dependencies.len(), 
             cargo_info.dev_dependencies.len(), 
             cargo_info.build_dependencies.len());
    
    let total_deps = cargo_info.dependencies.len() + 
                    cargo_info.dev_dependencies.len() + 
                    cargo_info.build_dependencies.len();
    println!("• Total dependencies: {}", total_deps);
    
    if !cargo_info.features.is_empty() {
        println!("• Features: {}", cargo_info.features.len());
    }
    
    if !cargo_info.targets.is_empty() {
        println!("• Build targets: {}", cargo_info.targets.len());
    }
    
    if cargo_info.workspace.is_some() {
        println!("• Workspace: Yes");
    }
    
    println!("\n✅ CargoAnalyzer demonstration completed successfully!");
    println!("The analyzer successfully parsed and extracted all information from Cargo.toml");
    
    Ok(())
}