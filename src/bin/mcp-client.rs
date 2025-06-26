use clap::Parser;
use mcp::{
    ChatSession, McpClient, Ollama, OllamaConfig,
    ollama::{OllamaFunction, OllamaParameters, OllamaProperty, OllamaTool},
};

#[derive(Parser)]
#[command(name = "mcp-client")]
#[command(about = "An MCP client for Ollama API")]
#[command(version = "0.1.0")]
struct Args {
    /// Read prompt from a file
    #[arg(short = 'f', long = "prompt-file")]
    prompt_file: Option<String>,

    /// Start conversational mode
    #[arg(short = 'c', long = "converse")]
    converse: bool,

    /// Specify the model to use
    #[arg(short = 'm', long = "model", default_value = "llama3")]
    model: String,

    /// MCP server address (e.g., http://localhost:3000/mcp)
    #[arg(short = 's', long = "mcp-server", required = true)]
    mcp_server: String,
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Validate that either converse or prompt-file is provided
    if !args.converse && args.prompt_file.is_none() || args.converse && args.prompt_file.is_some() {
        eprintln!(
            "Error: You must provide either --converse (-c) or --prompt-file (-f), but not both"
        );
        eprintln!("Use --help for more information");
        std::process::exit(1);
    }

    if !args.converse {
        let prompt_file = args.prompt_file.unwrap(); // Safe due to validation above
        let msg = std::fs::read_to_string(&prompt_file).unwrap_or_else(|_| {
            eprintln!("Error reading prompt file: {}", prompt_file);
            std::process::exit(1);
        });
        println!("Using message from file: {}", msg.trim());
        let response = Ollama::default(args.model.clone())
            .chat(&msg, &args.model)
            .unwrap();
        println!("Response: {}", response.message.content);
        return;
    }

    // Initialize MCP client and get tool list
    println!("Connecting to MCP server: {}", args.mcp_server);
    let mcp_client = McpClient::new(&args.mcp_server);

    let tools = match mcp_client.list_tools() {
        Ok(tools) => {
            println!(
                "Successfully retrieved {} tools from MCP server",
                tools.len()
            );
            tools
        }
        Err(e) => {
            eprintln!("Failed to get tools from MCP server: {}", e);
            eprintln!(
                "Make sure the MCP server is running at: {}",
                args.mcp_server
            );
            std::process::exit(1);
        }
    };

    // Create Ollama configuration
    let config = OllamaConfig::new(&args.model)
        .temperature(0.7)
        .max_tokens(100);
    println!("Ollama config created: {:?}", config);

    if !tools.is_empty() {
        println!("Available tools that could be used by the LLM:");
        for tool in &tools {
            println!(
                "  - {}: {}",
                tool.name,
                tool.description.as_deref().unwrap_or("No description")
            );
        }
    }
    let ollama_tools = vec![OllamaTool::function(OllamaFunction::new(
        "file_read",
        "Read the contents of a file from the filesystem. The path must be within /tmp/allowed_files/",
        OllamaParameters::new()
            .add_property("path", OllamaProperty::string("The file path to read")),
    ))];

    let mut session = ChatSession::New(args.model, ollama_tools);
    println!("Starting conversational mode. Type 'quit' or 'exit' to stop.");
    println!("Type your message and press Enter:");

    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let message = input.trim();
                if message.is_empty() {
                    continue;
                }
                if message.eq_ignore_ascii_case("quit") || message.eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                match session.send(message) {
                    Ok(response) => {
                        println!("Assistant: {}", response.message.content);

                        // Handle tool calls if present
                        if let Some(tool_calls) = response.message.tool_calls {
                            for tool_call in tool_calls {
                                println!("Tool call: {}", tool_call.function.name);
                                println!("Tool call arguments: {}", tool_call.function.arguments);

                                // Execute the tool on the MCP server
                                match mcp_client.call_tool(
                                    &tool_call.function.name,
                                    Some(tool_call.function.arguments),
                                ) {
                                    Ok(tool_result) => {
                                        println!("Tool result: {}", tool_result);

                                        // Send the tool result back to the conversation
                                        let tool_result_message = format!(
                                            "Tool '{}' executed successfully. Result: {}",
                                            tool_call.function.name, tool_result
                                        );

                                        match session.send(&tool_result_message) {
                                            Ok(follow_up_response) => {
                                                println!(
                                                    "Assistant: {}",
                                                    follow_up_response.message.content
                                                );
                                            }
                                            Err(e) => {
                                                println!(
                                                    "Error sending tool result to assistant: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!(
                                            "Error executing tool '{}': {}",
                                            tool_call.function.name, e
                                        );

                                        // Send the error back to the conversation
                                        let error_message = format!(
                                            "Tool '{}' execution failed: {}",
                                            tool_call.function.name, e
                                        );

                                        match session.send(&error_message) {
                                            Ok(error_response) => {
                                                println!(
                                                    "Assistant: {}",
                                                    error_response.message.content
                                                );
                                            }
                                            Err(e) => {
                                                println!(
                                                    "Error sending tool error to assistant: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error making request to Ollama: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }
}
