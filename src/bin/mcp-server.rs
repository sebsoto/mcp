use mcp::file_read;
use mcp::mcp::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create a new MCP server
    let server = McpServer::new();

    // Add the file_read tool using the dedicated module
    server.add_tool(file_read::get_tool_definition()).await;

    println!("MCP server starting with file_read tool...");
    println!("You can test it with:");
    println!("curl -X POST http://localhost:8080/mcp \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"jsonrpc\":\"2.0\",\"id\":\"1\",\"method\":\"tools/list\"}}'");
    println!();
    println!("Or call the file_read tool:");
    println!("curl -X POST http://localhost:8080/mcp \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!(
        "  -d '{{\"jsonrpc\":\"2.0\",\"id\":\"2\",\"method\":\"tools/call\",\"params\":{{\"name\":\"file_read\",\"arguments\":{{\"path\":\"Cargo.toml\"}}}}}}'"
    );

    // Start the server (this will run indefinitely)
    server.start().await?;

    Ok(())
}
