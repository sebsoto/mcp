# MCP - Model Context Protocol

A Rust-based client/server application designed to streamline interactions with various large language models (LLMs) through a unified, context-aware framework.

### Notice

Large portions of this project were written by an LLM.
This project was used to learn the limits of LLM coding agents, and how they could work into my daily workflow.

## Overview

The **mcp-server** acts as a proxy or gateway, while the **mcp-client** provides a command-line interface to send requests to the server. This project demonstrates how to build modular, networked applications in Rust and provides a framework for composing and managing interactions with various AI models.

For more details on the protocol itself, visit [modelcontextprotocol.io](https://modelcontextprotocol.io).

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building the Application](#building-the-application)
- [Running the Applications](#running-the-applications)
- [Client Usage Examples](#client-usage-examples)

## Prerequisites

This project requires **Rust** for building the application and **Ollama** for serving the local language models.

### Ollama Setup

MCP requires a running LLM server to connect to. Ollama is a tool for running open-source LLMs locally.

#### Install Ollama
Follow the installation instructions on the [Ollama website](https://ollama.com).

#### Pull a Model
Download a model you want to use. For example, to get the `granite3.3` model:

```bash
ollama pull granite3.3
```

#### Ensure Ollama is Running
After installation, Ollama runs as a background server on `http://localhost:11434`. The mcp-server will connect to this address.

## Building the Application

Build the entire project (both client and server) using Cargo:

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

The executables will be placed in:
- Development: `target/debug/`
- Release: `target/release/`

## Running the Applications

The process involves three steps:

### 1. Start the LLM Server (Ollama)

Ensure Ollama is running on your computer. It typically starts automatically after installation.

### 2. Start the MCP Server

The mcp-server listens for requests from the client on port 8080 by default.

**Option A: Using cargo run**
```bash
cargo run --bin mcp-server
```

**Option B: Running the compiled binary**
```bash
# Debug build
./target/debug/mcp-server

# Release build
./target/release/mcp-server
```

You should see output similar to:

INFO server started listening on :8080

The server is now ready to accept requests at the localhost:8080/mcp endpoint.

### 3. Run the MCP Client

The client sends requests to the server. It can be run in two main modes:

- Enter an interactive chat session using the -c flag.
- Provide a filepath to a text prompt via the -f flag.

## Client Usage Examples

Here are examples of how to run the mcp-client in interactive chat mode, specifying the model and server URL as requested.

### Example Command

First, ensure both Ollama and the mcp-server are running. Then, run one of the following commands.

#### Option A: `Using cargo run`
(Note the -- which separates cargo run arguments from the application's arguments)
```bash
cargo run --bin mcp-client -- -c -m granite3.3:latest -s http://localhost:8080/mcp
```

#### Option B: Running the compiled binary
(Assuming a debug build)
```bash
./target/debug/mcp-client -c -m granite3.3:latest -s http://localhost:8080/mcp
```

After running the command, you will be in an interactive prompt. You can start typing your questions or commands for the granite3.3:latest model.

### Example Session:
```bash
$ cargo run --bin mcp-client -- -c -m granite3.3:latest -s http://localhost:8080/mcp
> Read /tmp/allowed_files/payload.txt
Assistant: 
Tool call: file_read
Tool call arguments: {"path":"/tmp/allowed_files/payload.txt"}
Tool result: {"content":[{"text":"File: /tmp/allowed_files/payload.txt\nSize: 14 bytes\nMIME Type: text/plain\n\nContent:\nYou found me!\n","type":"text"}]}
Assistant: The file /tmp/allowed_files/payload.txt has been read successfully. Here are the details: 

- File: /tmp/allowed_files/payload.txt
- Size: 14 bytes
- MIME Type: text/plain

Content:
'''
You found me!
'''
> Read /home/srq/t.txt
Assistant: 
Tool call: file_read
Tool call arguments: {"path":"/home/srq/t.txt"}
Tool result: {"content":[{"text":"Error reading file: Access denied: File path must be within /tmp/allowed_files/","type":"text"}]}
Assistant: I'm sorry, but I cannot read files outside of the allowed directory. The file at /home/srq/t.txt is not accessible due to restricted permissions. Please provide a valid path that resides within /tmp/allowed_files/.
> quit
Goodbye!
```
