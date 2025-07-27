use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use tracing::{error, info};

mod docker;
mod environment;
mod executor;
mod tests;

use docker::DockerManager;

#[derive(Parser)]
#[command(name = "qa")]
#[command(about = "QA testing tool for tree CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run QA tests on specified platforms
    Test {
        /// Platforms to test (linux, alpine, windows)
        #[arg(long, value_delimiter = ',')]
        platforms: Vec<String>,

        /// Run all platforms
        #[arg(long)]
        all: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Clean up Docker resources
    Clean {
        /// Force cleanup without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            platforms,
            all,
            verbose,
        } => {
            run_tests(platforms, all, verbose).await?;
        }
        Commands::Clean { force } => {
            clean_resources(force).await?;
        }
    }

    Ok(())
}

async fn run_tests(platforms: Vec<String>, all: bool, verbose: bool) -> Result<()> {
    println!("{}", "🔬 Tree CLI QA Testing".bright_blue().bold());

    let test_platforms = if all || platforms.is_empty() {
        vec!["linux".to_string(), "alpine".to_string()]
    } else {
        platforms
    };

    info!("Testing platforms: {:?}", test_platforms);

    for platform in test_platforms {
        println!(
            "\n{} {}",
            "🐳 Testing platform:".bright_green(),
            platform.bright_white().bold()
        );

        match test_platform(&platform, verbose).await {
            Ok(_) => println!("{} {} tests passed", "✅".green(), platform),
            Err(e) => {
                error!("Platform {} failed: {}", platform, e);
                println!("{} {} tests failed: {}", "❌".red(), platform, e);
            }
        }
    }

    Ok(())
}

async fn test_platform(platform: &str, verbose: bool) -> Result<()> {
    let docker = DockerManager::new().await?;

    // Get project root (go up from tools/qa to project root)
    let project_root = std::env::current_dir()?
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("../.."));

    // Build Docker image
    let image_name = docker.build_image(platform, &project_root).await?;

    // Run tests
    let results = docker.run_tests(&image_name, platform).await?;

    if verbose {
        println!("Test output:\n{}", results.output);
    }

    if results.success {
        println!(
            "  {} {} tests passed ({}/{})",
            "✅".green(),
            platform,
            results.passed_tests,
            results.total_tests
        );
    } else {
        println!(
            "  {} {} tests failed ({}/{} passed)",
            "❌".red(),
            platform,
            results.passed_tests,
            results.total_tests
        );
        return Err(anyhow::anyhow!("Tests failed"));
    }

    Ok(())
}

async fn clean_resources(force: bool) -> Result<()> {
    println!("{}", "🧹 Cleaning Docker resources".bright_yellow());

    if !force {
        println!("This will remove all tree-qa Docker containers and images.");
        println!("Continue? (y/N)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    }

    let docker = DockerManager::new().await?;
    docker.cleanup_resources().await?;
    println!("  {} Cleanup completed", "✅".green());

    Ok(())
}
