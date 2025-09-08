#!/bin/bash
# Setup script for Trading API MCP Server

set -e

echo "Setting up Trading API MCP Server..."

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install dependencies
echo "Installing Python dependencies..."
pip install --upgrade pip
pip install -r requirements.txt

# Make MCP server executable
chmod +x mcp_server.py

echo "Setup complete!"
echo ""
echo "To run the MCP server:"
echo "1. Start your Rust API server: cargo run"
echo "2. In another terminal, activate venv: source venv/bin/activate"
echo "3. Run the MCP server: python mcp_server.py"
echo ""
echo "The MCP server will communicate via stdio and can be used with"
echo "AI agents that support the Model Context Protocol."
