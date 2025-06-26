//! MCP (Model Context Protocol) client and server module
//!
//! This module provides functionality for communicating with MCP servers using JSON-RPC 2.0
//! and implementing MCP servers.

use axum::{Router, extract::State, http::StatusCode, response::Json, routing::post};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

/// JSON-RPC 2.0 request structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// MCP Tool definition
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub inputSchema: Option<Value>,
}

/// Response from tools/list request
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolsListResponse {
    pub tools: Vec<McpTool>,
}

/// Request for tools/call endpoint
#[derive(Debug, Deserialize)]
pub struct ToolsCallRequest {
    pub name: String,
    pub arguments: Option<Value>,
}

/// Response from tools/call request
#[derive(Debug, Serialize)]
pub struct ToolsCallResponse {
    pub content: Vec<ToolContent>,
}

/// Tool execution result content
#[derive(Debug, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP Client for communicating with MCP servers
pub struct McpClient {
    base_url: String,
    client: Client,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Generate a unique request ID
    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Make a JSON-RPC request to the MCP server
    pub fn make_request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<JsonRpcResponse, Box<dyn std::error::Error>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Self::generate_id(),
            method: method.to_string(),
            params,
        };

        let response = self.client.post(&self.base_url).json(&request).send()?;

        if response.status().is_success() {
            let json_response: JsonRpcResponse = response.json()?;

            if let Some(error) = &json_response.error {
                return Err(format!("MCP server error {}: {}", error.code, error.message).into());
            }

            Ok(json_response)
        } else {
            let status = response.status();
            let error_text = response.text()?;
            Err(format!("HTTP error {}: {}", status, error_text).into())
        }
    }

    /// Get the list of available tools from the MCP server
    pub fn list_tools(&self) -> Result<Vec<McpTool>, Box<dyn std::error::Error>> {
        println!("Requesting tool list from MCP server: {}", self.base_url);

        let response = self.make_request("tools/list", None)?;

        if let Some(result) = response.result {
            let tools_response: ToolsListResponse = serde_json::from_value(result)?;
            println!(
                "Retrieved {} tools from MCP server",
                tools_response.tools.len()
            );

            for tool in &tools_response.tools {
                println!(
                    "  - Tool: {} - {}",
                    tool.name,
                    tool.description.as_deref().unwrap_or("No description")
                );
            }

            Ok(tools_response.tools)
        } else {
            Err("No result in tools/list response".into())
        }
    }

    /// Call a specific tool on the MCP server
    pub fn call_tool(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let response = self.make_request("tools/call", Some(params))?;

        if let Some(result) = response.result {
            Ok(result)
        } else {
            Err("No result in tools/call response".into())
        }
    }
}

/// MCP Server state containing registered tools
#[derive(Debug, Clone)]
pub struct McpServerState {
    pub tools: Arc<RwLock<HashMap<String, McpTool>>>,
}

impl McpServerState {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a tool to the server
    pub async fn add_tool(&self, tool: McpTool) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), tool);
    }

    /// Get all tools
    pub async fn get_tools(&self) -> Vec<McpTool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Get a specific tool by name
    pub async fn get_tool(&self, name: &str) -> Option<McpTool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }
}

/// Handle JSON-RPC requests
async fn handle_jsonrpc(
    State(state): State<McpServerState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    let response = match request.method.as_str() {
        "tools/list" => {
            let tools = state.get_tools().await;
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(ToolsListResponse { tools }).unwrap()),
                error: None,
            }
        }
        "tools/call" => {
            if let Some(params) = request.params {
                match serde_json::from_value::<ToolsCallRequest>(params) {
                    Ok(call_request) => {
                        match state.get_tool(&call_request.name).await {
                            Some(_tool) => {
                                // Handle specific tools
                                match call_request.name.as_str() {
                                    "file_read" => {
                                        if let Some(arguments) = call_request.arguments {
                                            match serde_json::from_value::<
                                                crate::file_read::FileReadRequest,
                                            >(
                                                arguments
                                            ) {
                                                Ok(file_request) => {
                                                    match crate::file_read::execute_file_read(
                                                        file_request,
                                                    ) {
                                                        Ok(file_response) => {
                                                            let result = ToolsCallResponse {
                                                                content: vec![ToolContent {
                                                                    content_type: "text"
                                                                        .to_string(),
                                                                    text: format!(
                                                                        "File: {}\nSize: {} bytes\nMIME Type: {}\n\nContent:\n{}",
                                                                        file_response.path,
                                                                        file_response.size,
                                                                        file_response
                                                                            .mime_type
                                                                            .as_deref()
                                                                            .unwrap_or("unknown"),
                                                                        file_response.content
                                                                    ),
                                                                }],
                                                            };
                                                            JsonRpcResponse {
                                                                jsonrpc: "2.0".to_string(),
                                                                id: request.id,
                                                                result: Some(
                                                                    serde_json::to_value(result)
                                                                        .unwrap(),
                                                                ),
                                                                error: None,
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let result = ToolsCallResponse {
                                                                content: vec![ToolContent {
                                                                    content_type: "text"
                                                                        .to_string(),
                                                                    text: format!(
                                                                        "Error reading file: {}",
                                                                        e
                                                                    ),
                                                                }],
                                                            };
                                                            JsonRpcResponse {
                                                                jsonrpc: "2.0".to_string(),
                                                                id: request.id,
                                                                result: Some(
                                                                    serde_json::to_value(result)
                                                                        .unwrap(),
                                                                ),
                                                                error: None,
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    id: request.id,
                                                    result: None,
                                                    error: Some(JsonRpcError {
                                                        code: -32602,
                                                        message: format!(
                                                            "Invalid file_read arguments: {}",
                                                            e
                                                        ),
                                                        data: None,
                                                    }),
                                                },
                                            }
                                        } else {
                                            JsonRpcResponse {
                                                jsonrpc: "2.0".to_string(),
                                                id: request.id,
                                                result: None,
                                                error: Some(JsonRpcError {
                                                    code: -32602,
                                                    message: "file_read tool requires arguments"
                                                        .to_string(),
                                                    data: None,
                                                }),
                                            }
                                        }
                                    }
                                    _ => {
                                        // Generic tool response for unknown tools
                                        let result = ToolsCallResponse {
                                            content: vec![ToolContent {
                                                content_type: "text".to_string(),
                                                text: format!(
                                                    "Tool '{}' executed successfully with arguments: {:?}",
                                                    call_request.name, call_request.arguments
                                                ),
                                            }],
                                        };
                                        JsonRpcResponse {
                                            jsonrpc: "2.0".to_string(),
                                            id: request.id,
                                            result: Some(serde_json::to_value(result).unwrap()),
                                            error: None,
                                        }
                                    }
                                }
                            }
                            None => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32601,
                                    message: format!("Tool '{}' not found", call_request.name),
                                    data: None,
                                }),
                            },
                        }
                    }
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Invalid params: {}", e),
                            data: None,
                        }),
                    },
                }
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    }),
                }
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    };

    Ok(Json(response))
}

/// MCP Server that handles JSON-RPC requests
pub struct McpServer {
    state: McpServerState,
    port: u16,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        Self {
            state: McpServerState::new(),
            port: 8080,
        }
    }

    /// Create a new MCP server with custom port
    pub fn with_port(port: u16) -> Self {
        Self {
            state: McpServerState::new(),
            port,
        }
    }

    /// Add a tool to the server
    pub async fn add_tool(&self, tool: McpTool) {
        self.state.add_tool(tool).await;
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = Router::new()
            .route("/mcp", post(handle_jsonrpc))
            .layer(CorsLayer::permissive())
            .with_state(self.state.clone());

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;

        println!("Starting MCP server on http://localhost:{}/mcp", self.port);

        axum::serve(listener, app).await?;

        Ok(())
    }
}
