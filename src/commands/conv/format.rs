//! Output formatting functionality for conversations

use crate::api::ApiResponse;
use std::fmt;

/// Output format for conversation list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Jsonl,
    Table,
    Tsv,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "jsonl" => Ok(OutputFormat::Jsonl),
            "table" => Ok(OutputFormat::Table),
            "tsv" => Ok(OutputFormat::Tsv),
            _ => Err(format!(
                "Invalid format '{}'. Valid values: json, jsonl, table, tsv",
                s
            )),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Jsonl => write!(f, "jsonl"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Tsv => write!(f, "tsv"),
        }
    }
}

/// Format response for output
pub fn format_response(response: &ApiResponse, format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&response)
            .map_err(|e| format!("Failed to serialize JSON: {}", e)),
        OutputFormat::Jsonl => {
            if let Some(channels) = response.data.get("channels") {
                if let Some(channels_array) = channels.as_array() {
                    let lines: Vec<String> = channels_array
                        .iter()
                        .filter_map(|conv| serde_json::to_string(conv).ok())
                        .collect();
                    Ok(lines.join("\n"))
                } else {
                    Ok(String::new())
                }
            } else {
                Ok(String::new())
            }
        }
        OutputFormat::Table => format_as_table(response),
        OutputFormat::Tsv => format_as_tsv(response),
    }
}

/// Format response as table
fn format_as_table(response: &ApiResponse) -> Result<String, String> {
    let channels = match response.data.get("channels").and_then(|v| v.as_array()) {
        Some(ch) => ch,
        None => return Ok(String::new()),
    };

    if channels.is_empty() {
        return Ok(String::new());
    }

    // Calculate column widths
    let mut max_id = "ID".len();
    let mut max_name = "NAME".len();
    let max_private = "PRIVATE".len();
    let max_member = "MEMBER".len();
    let mut max_num_members = "NUM_MEMBERS".len();

    for conv in channels {
        if let Some(id) = conv.get("id").and_then(|v| v.as_str()) {
            max_id = max_id.max(id.len());
        }
        if let Some(name) = conv.get("name").and_then(|v| v.as_str()) {
            max_name = max_name.max(name.len());
        }
        if let Some(num) = conv.get("num_members").and_then(|v| v.as_i64()) {
            max_num_members = max_num_members.max(num.to_string().len());
        }
    }

    // Build header
    let mut output = String::new();
    output.push_str(&format!(
        "{:width_id$}  {:width_name$}  {:width_private$}  {:width_member$}  {:width_num$}\n",
        "ID",
        "NAME",
        "PRIVATE",
        "MEMBER",
        "NUM_MEMBERS",
        width_id = max_id,
        width_name = max_name,
        width_private = max_private,
        width_member = max_member,
        width_num = max_num_members,
    ));

    // Build separator
    output.push_str(&format!(
        "{}  {}  {}  {}  {}\n",
        "-".repeat(max_id),
        "-".repeat(max_name),
        "-".repeat(max_private),
        "-".repeat(max_member),
        "-".repeat(max_num_members),
    ));

    // Build rows
    for conv in channels {
        let id = conv.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = conv.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let is_private = conv
            .get("is_private")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_member = conv
            .get("is_member")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let num_members = conv.get("num_members").and_then(|v| v.as_i64());

        let num_members_str = num_members.map(|n| n.to_string()).unwrap_or_default();

        output.push_str(&format!(
            "{:width_id$}  {:width_name$}  {:width_private$}  {:width_member$}  {:width_num$}\n",
            id,
            name,
            is_private,
            is_member,
            num_members_str,
            width_id = max_id,
            width_name = max_name,
            width_private = max_private,
            width_member = max_member,
            width_num = max_num_members,
        ));
    }

    Ok(output)
}

/// Format response as TSV
fn format_as_tsv(response: &ApiResponse) -> Result<String, String> {
    let channels = match response.data.get("channels").and_then(|v| v.as_array()) {
        Some(ch) => ch,
        None => return Ok(String::new()),
    };

    if channels.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::new();

    // Header
    output.push_str("id\tname\tis_private\tis_member\tnum_members\n");

    // Rows
    for conv in channels {
        let id = conv.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = conv.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let is_private = conv
            .get("is_private")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_member = conv
            .get("is_member")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let num_members = conv.get("num_members").and_then(|v| v.as_i64());

        let num_members_str = num_members.map(|n| n.to_string()).unwrap_or_default();

        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            id, name, is_private, is_member, num_members_str
        ));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_output_format_parse() {
        assert_eq!(OutputFormat::parse("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("jsonl").unwrap(), OutputFormat::Jsonl);
        assert_eq!(OutputFormat::parse("table").unwrap(), OutputFormat::Table);
        assert_eq!(OutputFormat::parse("tsv").unwrap(), OutputFormat::Tsv);
        assert!(OutputFormat::parse("invalid").is_err());
    }

    #[test]
    fn test_format_response_jsonl() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general"},
                    {"id": "C2", "name": "random"},
                ]),
            )]),
            error: None,
        };

        let output = format_response(&response, OutputFormat::Jsonl).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"id\":\"C1\""));
        assert!(lines[1].contains("\"id\":\"C2\""));
    }

    #[test]
    fn test_format_response_tsv() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general", "is_private": false, "is_member": true, "num_members": 42},
                    {"id": "C2", "name": "private", "is_private": true, "is_member": false},
                ]),
            )]),
            error: None,
        };

        let output = format_response(&response, OutputFormat::Tsv).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert_eq!(lines[0], "id\tname\tis_private\tis_member\tnum_members");
        assert_eq!(lines[1], "C1\tgeneral\tfalse\ttrue\t42");
        assert_eq!(lines[2], "C2\tprivate\ttrue\tfalse\t"); // num_members missing -> empty
    }

    #[test]
    fn test_format_response_table() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general", "is_private": false, "is_member": true, "num_members": 42},
                ]),
            )]),
            error: None,
        };

        let output = format_response(&response, OutputFormat::Table).unwrap();
        assert!(output.contains("ID"));
        assert!(output.contains("NAME"));
        assert!(output.contains("PRIVATE"));
        assert!(output.contains("MEMBER"));
        assert!(output.contains("NUM_MEMBERS"));
        assert!(output.contains("C1"));
        assert!(output.contains("general"));
        assert!(output.contains("42"));
    }
}
