//! File reading tool module
//!
//! This module provides functionality for reading files from the filesystem
//! as part of the MCP server's tool capabilities.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// File read tool request parameters
#[derive(Debug, Deserialize)]
pub struct FileReadRequest {
    pub path: String,
}

/// File read tool response
#[derive(Debug, Serialize)]
pub struct FileReadResponse {
    pub content: String,
    pub path: String,
    pub size: usize,
    pub mime_type: Option<String>,
}

/// Execute the file read tool
pub fn execute_file_read(
    request: FileReadRequest,
) -> Result<FileReadResponse, Box<dyn std::error::Error>> {
    let path = Path::new(&request.path);
    if !request.path.starts_with("/tmp/allowed_files/") {
        return Err("Access denied: File path must be within /tmp/allowed_files/".into());
    }

    // Validate the path
    if !path.exists() {
        return Err(format!("File not found: {}", request.path).into());
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", request.path).into());
    }

    // Read the file content
    match fs::read_to_string(path) {
        Ok(content) => {
            let size = content.len();
            let mime_type = guess_mime_type(&request.path);

            Ok(FileReadResponse {
                content,
                path: request.path,
                size,
                mime_type,
            })
        }
        Err(error) => Err(format!("Failed to read file '{}': {}", request.path, error).into()),
    }
}

/// Simple MIME type guessing based on file extension
fn guess_mime_type(path: &str) -> Option<String> {
    let path = Path::new(path);
    match path.extension()?.to_str()? {
        "txt" => Some("text/plain".to_string()),
        "md" => Some("text/markdown".to_string()),
        "rs" => Some("text/x-rust".to_string()),
        "py" => Some("text/x-python".to_string()),
        "js" => Some("text/javascript".to_string()),
        "ts" => Some("text/typescript".to_string()),
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "html" => Some("text/html".to_string()),
        "css" => Some("text/css".to_string()),
        "yaml" | "yml" => Some("application/x-yaml".to_string()),
        "toml" => Some("application/toml".to_string()),
        _ => None,
    }
}

/// Get the tool definition for the file_read tool
pub fn get_tool_definition() -> crate::mcp::McpTool {
    crate::mcp::McpTool {
        name: "file_read".to_string(),
        description: Some("Read the contents of a file from the filesystem. The path must be within /tmp/allowed_files/".to_string()),
        inputSchema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The file path to read"
                }
            },
            "required": ["path"]
        })),
    }
}
