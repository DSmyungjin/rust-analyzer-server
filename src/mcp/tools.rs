use crate::protocol::mcp::ToolDefinition;
use serde_json::json;

pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        // 1. Must be first - workspace setup
        ToolDefinition {
            name: "rust_analyzer_set_workspace".to_string(),
            description: "Set the workspace root directory for rust-analyzer".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workspace_path": { "type": "string", "description": "Path to the workspace root" }
                },
                "required": ["workspace_path"]
            }),
        },
        // 2-5. Most frequently used - core navigation
        ToolDefinition {
            name: "rust_analyzer_hover".to_string(),
            description: "Get hover information for a symbol at a specific position in a Rust file"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_definition".to_string(),
            description: "Go to definition of a symbol at a specific position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_references".to_string(),
            description: "Find all references to a symbol at a specific position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_workspace_symbol".to_string(),
            description: "Search for symbols across the entire workspace using a query (supports fuzzy matching)"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query for symbol names (e.g., 'TradeData', 'calculate')"
                    }
                },
                "required": ["query"]
            }),
        },
        // 6-8. Frequently used
        ToolDefinition {
            name: "rust_analyzer_diagnostics".to_string(),
            description: "Get compiler diagnostics (errors, warnings, hints) for a Rust file"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_inlay_hint".to_string(),
            description: "Get inlay hints (type annotations) for a code range".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Start line number (0-based)" },
                    "character": { "type": "number", "description": "Start character position (0-based)" },
                    "end_line": { "type": "number", "description": "End line number (0-based)" },
                    "end_character": { "type": "number", "description": "End character position (0-based)" }
                },
                "required": ["file_path", "line", "character", "end_line", "end_character"]
            }),
        },
        // 8-11. Code structure analysis
        ToolDefinition {
            name: "rust_analyzer_implementation".to_string(),
            description: "Find all implementations of a trait at a specific position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_incoming_calls".to_string(),
            description: "Find all functions that call this function".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_outgoing_calls".to_string(),
            description: "Find all functions that this function calls".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        ToolDefinition {
            name: "rust_analyzer_parent_module".to_string(),
            description: "Navigate to parent module declaration".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Rust file" },
                    "line": { "type": "number", "description": "Line number (0-based)" },
                    "character": { "type": "number", "description": "Character position (0-based)" }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
        // 12. Workspace-wide utilities
        ToolDefinition {
            name: "rust_analyzer_workspace_diagnostics".to_string(),
            description: "Get all compiler diagnostics across the entire workspace".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}
