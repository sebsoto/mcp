//! Ollama library module
//!
//! This module provides functionality for working with Ollama models.

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A single message in a chat conversation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OllamaToolCall>>,
}

impl ChatMessage {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create a new assistant message with tool calls
    pub fn assistant_with_tools(
        content: impl Into<String>,
        tool_calls: Vec<OllamaToolCall>,
    ) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: Some(tool_calls),
        }
    }
}

/// Tool definition for Ollama API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OllamaFunction,
}

impl OllamaTool {
    /// Create a new function tool
    pub fn function(function: OllamaFunction) -> Self {
        Self {
            tool_type: "function".to_string(),
            function,
        }
    }
}

/// Function definition for Ollama tools
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaFunction {
    pub name: String,
    pub description: String,
    pub parameters: OllamaParameters,
}

impl OllamaFunction {
    /// Create a new function definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: OllamaParameters,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// Parameters schema for Ollama functions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, OllamaProperty>,
    pub required: Vec<String>,
}

impl OllamaParameters {
    /// Create a new parameters schema
    pub fn new() -> Self {
        Self {
            param_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    /// Add a property to the parameters
    pub fn add_property(mut self, name: impl Into<String>, property: OllamaProperty) -> Self {
        self.properties.insert(name.into(), property);
        self
    }

    /// Add a required property
    pub fn add_required(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }
}

/// Property definition for Ollama parameters
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
}

impl OllamaProperty {
    /// Create a string property
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            prop_type: "string".to_string(),
            description: description.into(),
            r#enum: None,
        }
    }

    /// Create a string property with enum values
    pub fn string_enum(description: impl Into<String>, values: Vec<String>) -> Self {
        Self {
            prop_type: "string".to_string(),
            description: description.into(),
            r#enum: Some(values),
        }
    }

    /// Create a number property
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            prop_type: "number".to_string(),
            description: description.into(),
            r#enum: None,
        }
    }

    /// Create a boolean property
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            prop_type: "boolean".to_string(),
            description: description.into(),
            r#enum: None,
        }
    }
}

/// Tool call from Ollama response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaToolCall {
    pub function: OllamaFunctionCall,
}

/// Function call from Ollama response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaFunctionCall {
    pub name: String,
    pub arguments: Value,
}

/// Request payload for the /api/chat endpoint
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<OllamaTool>,
    pub stream: bool,
}

/// Response from the /api/chat endpoint
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Main Ollama client struct
pub struct Ollama {
    base_url: String,
    client: Client,
    // Fields copied from OllamaConfig
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    tools: Vec<OllamaTool>,
}

impl Ollama {
    /// Create a new Ollama client instance with configuration
    pub fn new(config: OllamaConfig, tools: Vec<OllamaTool>) -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: Client::new(),
            // Copy fields from OllamaConfig
            model: config.model,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            tools,
        }
    }
    pub fn default(model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: Client::new(),
            model: model.into(),
            temperature: None,
            max_tokens: None,
            tools: Vec::new(),
        }
    }

    /// Generate a completion using the Ollama API
    ///
    /// # Arguments
    /// * `message` - The message content to send to the model
    /// * `model` - The name of the model to use (e.g., "llama3", "mistral")
    ///
    /// # Returns
    /// Result containing the ChatResponse or an error
    pub fn chat(
        &self,
        message: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let user_message = ChatMessage::user(message);

        let request_payload = ChatRequest {
            model: model.into(),
            messages: vec![user_message],
            tools: self.tools.clone(), // No tools by default
            stream: false,             // Disable streaming for simplicity
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = self.client.post(&url).json(&request_payload).send()?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response.json()?;

            Ok(chat_response)
        } else {
            let error_text = response.text()?;
            Err(format!("Request failed with status : {}", error_text).into())
        }
    }

    /// Generate a completion using the Ollama API with tools
    ///
    /// # Arguments
    /// * `message` - The message content to send to the model
    /// * `model` - The name of the model to use (e.g., "llama3", "mistral")
    /// * `tools` - List of tools available to the model
    ///
    /// # Returns
    /// Result containing the ChatResponse or an error
    pub fn chat_with_tools(
        &self,
        message: impl Into<String>,
        model: impl Into<String>,
        tools: Vec<OllamaTool>,
    ) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let system_message = ChatMessage::system(
            "You are Granite, developed by IBM. You are a helpful assistant with tools. When a tool is required to answer the user's query, respond only with <|tool_call|> followed by a JSON list of tools used. If a tool does not exist in the provided list of tools, notify the user that you do not have the ability to fulfill the request.<|end_of_text|>",
        );
        let user_message = ChatMessage::user(message);

        let request_payload = ChatRequest {
            model: model.into(),
            messages: vec![system_message, user_message],
            tools,
            stream: false, // Disable streaming for simplicity
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = self.client.post(&url).json(&request_payload).send()?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response.json()?;

            Ok(chat_response)
        } else {
            let error_text = response.text()?;
            Err(format!("Request failed with status : {}", error_text).into())
        }
    }
}

/// Configuration for Ollama requests
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl OllamaConfig {
    /// Create a new configuration with the specified model
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            temperature: None,
            max_tokens: None,
        }
    }

    /// Set the temperature for generation
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum number of tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// Chat session that maintains conversation history
pub struct ChatSession {
    client: Client,
    pub base_url: String,
    pub model: String,
    tools: Vec<OllamaTool>,
    messages: Vec<ChatMessage>,
}

impl ChatSession {
    /// Create a new chat session
    pub fn New(model: impl Into<String>, tools: Vec<OllamaTool>) -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
            model: model.into(),
            tools,
            messages: Vec::new(),
        }
    }

    /// Send a message and maintain chat history
    ///
    /// # Arguments
    /// * `message` - The message content to send to the model
    ///
    /// # Returns
    /// Result containing the ChatResponse or an error
    pub fn send(
        &mut self,
        message: impl Into<String>,
    ) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let user_message = ChatMessage::user(message);
        self.messages.push(user_message);

        let request_payload = ChatRequest {
            model: self.model.clone(),
            messages: self.messages.clone(),
            tools: self.tools.clone(),
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = self.client.post(&url).json(&request_payload).send()?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response.json()?;

            // Add the assistant's response to the message history
            self.messages.push(chat_response.message.clone());

            Ok(chat_response)
        } else {
            let error_text = response.text()?;
            Err(format!("Request failed with status : {}", error_text).into())
        }
    }

    /// Add a system message to the conversation
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        let system_message = ChatMessage::system(content);
        self.messages.push(system_message);
    }
}
