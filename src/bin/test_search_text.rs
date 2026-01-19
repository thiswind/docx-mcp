use docx_mcp::docx_tools::DocxToolsProvider;
use docx_mcp::security::SecurityConfig;
use mcp_core::types::ToolResponseContent;
use serde_json::json;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing DocxToolsProvider search_text functionality ===\n");
    
    // Create provider
    let provider = DocxToolsProvider::new_with_security(SecurityConfig::default());
    
    // Test document path
    let test_doc_path = PathBuf::from("/Users/thiswind/Documents/工作/信息学院工作/桌面文件夹/cursor_workspaces/03_Program_Development/docx-mcp/专家审查意见表-王红梅.docx");
    
    if !test_doc_path.exists() {
        eprintln!("Error: Test document not found at {:?}", test_doc_path);
        return Ok(());
    }
    
    println!("1. Opening document: {:?}", test_doc_path);
    let open_result = provider.call_tool("open_document", json!({
        "path": test_doc_path.to_str().unwrap()
    })).await;
    
    let doc_id = match open_result.content.get(0) {
        Some(ToolResponseContent::Text(t)) => {
            let json_val: serde_json::Value = serde_json::from_str(&t.text)?;
            if json_val["success"].as_bool().unwrap_or(false) {
                let id = json_val["document_id"].as_str().unwrap();
                println!("   ✓ Document opened successfully, ID: {}", id);
                id.to_string()
            } else {
                eprintln!("   ✗ Failed to open document: {}", json_val["error"].as_str().unwrap_or("Unknown error"));
                return Ok(());
            }
        }
        _ => {
            eprintln!("   ✗ Unexpected response format");
            return Ok(());
        }
    };
    
    println!("\n2. Testing extract_text...");
    let extract_result = provider.call_tool("extract_text", json!({
        "document_id": doc_id
    })).await;
    
    match extract_result.content.get(0) {
        Some(ToolResponseContent::Text(t)) => {
            let json_val: serde_json::Value = serde_json::from_str(&t.text)?;
            if json_val["success"].as_bool().unwrap_or(false) {
                let text = json_val["text"].as_str().unwrap_or("");
                println!("   ✓ Text extracted successfully ({} characters)", text.len());
                // Safely get preview - use char_indices to avoid splitting multi-byte characters
                let preview: String = text.chars().take(200).collect();
                if !preview.is_empty() {
                    println!("   Preview (first 200 chars): {}", preview);
                }
            } else {
                eprintln!("   ✗ Failed to extract text: {}", json_val["error"].as_str().unwrap_or("Unknown error"));
            }
        }
        _ => {
            eprintln!("   ✗ Unexpected response format");
        }
    }
    
    println!("\n3. Testing search_text with '专家'...");
    let search_result = provider.call_tool("search_text", json!({
        "document_id": doc_id,
        "search_term": "专家",
        "case_sensitive": false
    })).await;
    
    match search_result.content.get(0) {
        Some(ToolResponseContent::Text(t)) => {
            let json_val: serde_json::Value = serde_json::from_str(&t.text)?;
            if json_val["success"].as_bool().unwrap_or(false) {
                let total_matches = json_val["total_matches"].as_u64().unwrap_or(0);
                println!("   ✓ Search completed successfully");
                println!("   Total matches: {}", total_matches);
                if let Some(matches) = json_val["matches"].as_array() {
                    for (i, m) in matches.iter().take(3).enumerate() {
                        println!("   Match {}: position {}, line {}", 
                            i + 1,
                            m["position"].as_u64().unwrap_or(0),
                            m["line"].as_u64().unwrap_or(0)
                        );
                        if let Some(ctx) = m["context"].as_str() {
                            // Safely get preview - use chars to avoid splitting multi-byte characters
                            let ctx_preview: String = ctx.chars().take(100).collect();
                            if !ctx_preview.is_empty() {
                                println!("     Context: {}", ctx_preview);
                            }
                        }
                    }
                }
            } else {
                eprintln!("   ✗ Search failed: {}", json_val["error"].as_str().unwrap_or("Unknown error"));
            }
        }
        _ => {
            eprintln!("   ✗ Unexpected response format");
        }
    }
    
    println!("\n4. Testing search_text with '审查'...");
    let search_result2 = provider.call_tool("search_text", json!({
        "document_id": doc_id,
        "search_term": "审查",
        "case_sensitive": false
    })).await;
    
    match search_result2.content.get(0) {
        Some(ToolResponseContent::Text(t)) => {
            let json_val: serde_json::Value = serde_json::from_str(&t.text)?;
            if json_val["success"].as_bool().unwrap_or(false) {
                let total_matches = json_val["total_matches"].as_u64().unwrap_or(0);
                println!("   ✓ Search completed successfully");
                println!("   Total matches: {}", total_matches);
            } else {
                eprintln!("   ✗ Search failed: {}", json_val["error"].as_str().unwrap_or("Unknown error"));
            }
        }
        _ => {
            eprintln!("   ✗ Unexpected response format");
        }
    }
    
    println!("\n5. Testing get_word_count...");
    let word_count_result = provider.call_tool("get_word_count", json!({
        "document_id": doc_id
    })).await;
    
    match word_count_result.content.get(0) {
        Some(ToolResponseContent::Text(t)) => {
            let json_val: serde_json::Value = serde_json::from_str(&t.text)?;
            if json_val["success"].as_bool().unwrap_or(false) {
                println!("   ✓ Word count retrieved successfully");
                if let Some(stats) = json_val["statistics"].as_object() {
                    println!("   Words: {}", stats.get("words").and_then(|v| v.as_u64()).unwrap_or(0));
                    println!("   Characters: {}", stats.get("characters").and_then(|v| v.as_u64()).unwrap_or(0));
                }
            } else {
                eprintln!("   ✗ Failed: {}", json_val["error"].as_str().unwrap_or("Unknown error"));
            }
        }
        _ => {
            eprintln!("   ✗ Unexpected response format");
        }
    }
    
    println!("\n6. Testing get_tables...");
    let tables_result = provider.call_tool("get_tables", json!({
        "document_id": doc_id
    })).await;
    
    match tables_result.content.get(0) {
        Some(ToolResponseContent::Text(t)) => {
            let json_val: serde_json::Value = serde_json::from_str(&t.text)?;
            if json_val["success"].as_bool().unwrap_or(false) {
                println!("   ✓ Tables retrieved successfully");
                if let Some(tables) = json_val["metadata"].get("tables").and_then(|v| v.as_array()) {
                    println!("   Number of tables: {}", tables.len());
                }
            } else {
                eprintln!("   ✗ Failed: {}", json_val["error"].as_str().unwrap_or("Unknown error"));
            }
        }
        _ => {
            eprintln!("   ✗ Unexpected response format");
        }
    }
    
    println!("\n=== Test completed ===");
    Ok(())
}
