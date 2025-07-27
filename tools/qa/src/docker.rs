use anyhow::{anyhow, Result};
use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions};
use bollard::image::BuildImageOptions;
use bollard::Docker;
use colored::*;
use std::path::Path;
use tracing::{info, warn};
use futures::StreamExt;

pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    pub async fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        
        // Test connection
        match docker.ping().await {
            Ok(_) => info!("Connected to Docker daemon"),
            Err(e) => return Err(anyhow!("Failed to connect to Docker: {}", e)),
        }

        Ok(Self { docker })
    }

    pub async fn build_image(&self, platform: &str, project_root: &Path) -> Result<String> {
        let image_name = format!("tree-qa:{}", platform);
        let dockerfile_path = format!("tests/qa/Dockerfile.{}", platform);
        
        println!("  {} Building Docker image: {}", "🔨".yellow(), image_name.bright_white());
        
        // Check if Dockerfile exists
        let dockerfile_full_path = project_root.join(&dockerfile_path);
        if !dockerfile_full_path.exists() {
            return Err(anyhow!("Dockerfile not found: {}", dockerfile_full_path.display()));
        }

        // Build options
        let build_options = BuildImageOptions {
            dockerfile: dockerfile_path,
            t: image_name.clone(),
            pull: true,
            nocache: true,
            ..Default::default()
        };

        // Create build context (tar stream)
        let tar = tar_directory(project_root)?;
        
        // Build image
        let mut stream = self.docker.build_image(build_options, None, Some(tar.into()));
        
        while let Some(build_result) = stream.next().await {
            match build_result {
                Ok(output) => {
                    if let Some(stream) = output.stream {
                        if stream.trim().len() > 0 {
                            print!("    {}", stream);
                        }
                    }
                    if let Some(error) = output.error {
                        return Err(anyhow!("Docker build error: {}", error));
                    }
                }
                Err(e) => return Err(anyhow!("Build stream error: {}", e)),
            }
        }

        println!("  {} Image built successfully", "✅".green());
        Ok(image_name)
    }

    pub async fn run_tests(&self, image_name: &str, platform: &str) -> Result<TestResults> {
        let container_name = format!("tree-qa-{}-{}", platform, chrono::Utc::now().timestamp());
        
        println!("  {} Creating container: {}", "🐳".blue(), container_name);

        // Container configuration
        let config = Config {
            image: Some(image_name.to_string()),
            working_dir: Some("/app".to_string()),
            cmd: Some(vec!["/app/tests/qa/qa-docker-test.sh".to_string()]),
            ..Default::default()
        };

        // Create container
        let create_options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        self.docker.create_container(Some(create_options), config).await?;

        println!("  {} Running tests in container", "🧪".cyan());

        // Start container 
        self.docker.start_container(&container_name, None::<StartContainerOptions<String>>).await?;

        // Wait for container to finish and get logs
        let mut wait_stream = self.docker.wait_container::<String>(&container_name, None);
        let wait_result = wait_stream.next().await
            .unwrap_or(Ok(bollard::models::ContainerWaitResponse { 
                status_code: 1, 
                error: None 
            }))?;
        let exit_code = wait_result.status_code;

        // Get container logs
        let log_options = bollard::container::LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        };
        
        let mut log_stream = self.docker.logs(&container_name, Some(log_options));
        let mut output = String::new();
        
        while let Some(log_result) = log_stream.next().await {
            match log_result {
                Ok(log_output) => {
                    output.push_str(&format!("{}", log_output));
                }
                Err(e) => warn!("Error reading logs: {}", e),
            }
        }

        // Remove container
        let remove_options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        
        if let Err(e) = self.docker.remove_container(&container_name, Some(remove_options)).await {
            warn!("Failed to remove container {}: {}", container_name, e);
        }

        // Parse test results
        let results = parse_test_output(&output, exit_code == 0);
        
        Ok(results)
    }

    pub async fn cleanup_resources(&self) -> Result<()> {
        println!("  {} Removing tree-qa containers", "🗑️".red());
        
        // List and remove containers
        let containers = self.docker.list_containers::<String>(None).await?;
        for container in containers {
            if let Some(names) = container.names {
                for name in names {
                    if name.contains("tree-qa") {
                        println!("    Removing container: {}", name);
                        let remove_options = RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        };
                        if let Err(e) = self.docker.remove_container(&name.trim_start_matches('/'), Some(remove_options)).await {
                            warn!("Failed to remove container {}: {}", name, e);
                        }
                    }
                }
            }
        }

        println!("  {} Removing tree-qa images", "🗑️".red());
        
        // List and remove images
        let images = self.docker.list_images::<String>(None).await?;
        for image in images {
            for tag in &image.repo_tags {
                if tag.starts_with("tree-qa:") {
                    println!("    Removing image: {}", tag);
                    if let Err(e) = self.docker.remove_image(&tag, None, None).await {
                        warn!("Failed to remove image {}: {}", tag, e);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct TestResults {
    pub platform: String,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub success: bool,
    pub output: String,
}

fn tar_directory(path: &Path) -> Result<Vec<u8>> {
    let mut tar_data = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut tar_data);
        tar.append_dir_all(".", path)?;
        tar.finish()?;
    }
    Ok(tar_data)
}

fn parse_test_output(output: &str, success: bool) -> TestResults {
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    for line in output.lines() {
        if line.contains("✓ PASS:") {
            passed_tests += 1;
            total_tests += 1;
        } else if line.contains("✗ FAIL:") {
            failed_tests += 1;
            total_tests += 1;
        }
    }

    TestResults {
        platform: "unknown".to_string(),
        total_tests,
        passed_tests,
        failed_tests,
        success,
        output: output.to_string(),
    }
}