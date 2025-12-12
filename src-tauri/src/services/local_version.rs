use regex::Regex;
use std::process::Command;

pub fn get_version(command_name: &str, version_arg: Option<&str>) -> Result<String, String> {
    let arg = version_arg.unwrap_or("--version");

    let output = Command::new(command_name)
        .arg(arg)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", command_name, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_str = format!("{}{}", stdout, stderr);

    // Try to extract version number (supports multiple formats)
    let version_regex = Regex::new(r"(\d+\.\d+(?:\.\d+)?(?:-[\w.]+)?)")
        .map_err(|e| e.to_string())?;

    version_regex
        .captures(&output_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| format!("Could not parse version from: {}", output_str.trim()))
}
