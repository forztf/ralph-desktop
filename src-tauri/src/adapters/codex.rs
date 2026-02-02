use super::{
    apply_extended_path, apply_shell_env, command_for_cli, hide_console_window, resolve_cli_path,
    CliAdapter, CommandOptions, LineType, ParsedLine,
};
use serde_json::Value;
use crate::storage::models::CliType;
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub struct CodexAdapter {
    path: Option<String>,
}

impl CodexAdapter {
    pub fn new() -> Self {
        let path = resolve_cli_path("codex");
        Self { path }
    }

    fn exec_args(prompt: &str, options: CommandOptions) -> Vec<String> {
        let mut args = vec![
            "exec".to_string(),
            "--dangerously-bypass-approvals-and-sandbox".to_string(),
        ];
        if options.skip_git_repo_check {
            args.push("--skip-git-repo-check".to_string());
        }
        args.push(prompt.to_string());
        args
    }

    fn readonly_args(prompt: &str, options: CommandOptions) -> Vec<String> {
        let mut args = vec![
            "exec".to_string(),
            "--dangerously-bypass-approvals-and-sandbox".to_string(),
            "--json".to_string(),  // Output JSONL for parsing
        ];
        if options.skip_git_repo_check {
            args.push("--skip-git-repo-check".to_string());
        }
        args.push(prompt.to_string());
        args
    }

    fn build_exec_command(
        &self,
        prompt: &str,
        working_dir: &Path,
        readonly: bool,
        options: CommandOptions,
    ) -> Command {
        let exe = self.path.as_deref().unwrap_or("codex");
        let args = if readonly {
            Self::readonly_args(prompt, options)
        } else {
            Self::exec_args(prompt, options)
        };
        let mut cmd = command_for_cli(exe, &args, working_dir);
        apply_extended_path(&mut cmd);
        apply_shell_env(&mut cmd);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }
}

#[async_trait]
impl CliAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "Codex CLI"
    }

    fn cli_type(&self) -> CliType {
        CliType::Codex
    }

    fn is_installed(&self) -> bool {
        self.path.is_some()
    }

    fn get_path(&self) -> Option<String> {
        self.path.clone()
    }

    async fn version(&self) -> Option<String> {
        let exe = self.path.as_deref().unwrap_or("codex");
        let mut cmd = Command::new(exe);
        apply_extended_path(&mut cmd);
        apply_shell_env(&mut cmd);
        hide_console_window(&mut cmd);
        let output = cmd.arg("--version").output().await.ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    fn build_command(&self, prompt: &str, working_dir: &Path, options: CommandOptions) -> Command {
        self.build_exec_command(prompt, working_dir, false, options)
    }

    fn build_readonly_command(
        &self,
        prompt: &str,
        working_dir: &Path,
        options: CommandOptions,
    ) -> Command {
        self.build_exec_command(prompt, working_dir, true, options)
    }

    fn detect_completion(&self, output: &str, signal: &str) -> bool {
        // Codex output is plain text, direct detection
        output.contains(signal)
    }

    fn parse_output_line(&self, line: &str) -> ParsedLine {
        // Codex --json outputs JSONL, parse item.completed events for agent_message text
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            let event_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");
            
            match event_type {
                "item.completed" => {
                    // Extract text from agent_message items
                    if let Some(item) = json.get("item") {
                        let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        if item_type == "agent_message" {
                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                return ParsedLine {
                                    content: text.to_string(),
                                    line_type: LineType::Json,
                                    is_assistant: true,
                                };
                            }
                        }
                    }
                    // Non-message item.completed, skip
                    ParsedLine {
                        content: String::new(),
                        line_type: LineType::Json,
                        is_assistant: false,
                    }
                }
                // Control events - skip silently
                "thread.started" | "turn.started" | "turn.completed" | "item.delta" |
                // Item lifecycle events - skip (we only care about item.completed with agent_message)
                "item.started" | "item.updated" |
                // Other item.completed types (reasoning, command_execution, file_change, etc.) - skip
                // They are handled by the item.completed branch above, non-agent_message returns empty
                // Meta events
                "session.started" | "session.completed" => {
                    ParsedLine {
                        content: String::new(),
                        line_type: LineType::Json,
                        is_assistant: false,
                    }
                }
                // Error events - log but don't pollute output
                "turn.failed" | "error" => {
                    // Extract error message for debugging, but return empty to avoid polluting brainstorm
                    let error_msg = json.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    eprintln!("[Codex] Error event: {}", error_msg);
                    ParsedLine {
                        content: String::new(),
                        line_type: LineType::Json,
                        is_assistant: false,
                    }
                }
                _ => {
                    // Unknown event type with a "type" field - skip to avoid polluting output
                    // This is safer than passing through as text
                    if !event_type.is_empty() {
                        eprintln!("[Codex] Skipping unknown event type: {}", event_type);
                        ParsedLine {
                            content: String::new(),
                            line_type: LineType::Json,
                            is_assistant: false,
                        }
                    } else {
                        // No type field - this might be direct JSON response (Loop mode or mock)
                        // Pass through as text for backward compatibility
                        ParsedLine {
                            content: line.to_string(),
                            line_type: LineType::Text,
                            is_assistant: true,
                        }
                    }
                }
            }
        } else {
            // Not JSON, treat as plain text (fallback for non --json mode)
            ParsedLine {
                content: line.to_string(),
                line_type: LineType::Text,
                is_assistant: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CodexAdapter;
    use crate::adapters::CliAdapter;
    use super::CommandOptions;

    #[test]
    fn exec_args_include_exec_and_full_auto() {
        let args = CodexAdapter::exec_args("hello", CommandOptions::default());
        assert_eq!(
            args,
            vec!["exec", "--dangerously-bypass-approvals-and-sandbox", "hello"]
        );
    }

    #[test]
    fn readonly_args_use_read_only_sandbox() {
        let args = CodexAdapter::readonly_args("hello", CommandOptions::default());
        assert_eq!(
            args,
            vec!["exec", "--dangerously-bypass-approvals-and-sandbox", "--json", "hello"]
        );
    }

    #[test]
    fn exec_args_include_skip_git_repo_check() {
        let args = CodexAdapter::exec_args(
            "hello",
            CommandOptions {
                skip_git_repo_check: true,
            },
        );
        assert_eq!(
            args,
            vec![
                "exec",
                "--dangerously-bypass-approvals-and-sandbox",
                "--skip-git-repo-check",
                "hello"
            ]
        );
    }

    #[test]
    fn readonly_args_include_skip_git_repo_check() {
        let args = CodexAdapter::readonly_args(
            "hello",
            CommandOptions {
                skip_git_repo_check: true,
            },
        );
        assert_eq!(
            args,
            vec![
                "exec",
                "--dangerously-bypass-approvals-and-sandbox",
                "--json",
                "--skip-git-repo-check",
                "hello"
            ]
        );
    }

    #[test]
    fn parse_output_line_extracts_agent_message_text() {
        let adapter = CodexAdapter::new();
        let line = r#"{"type":"item.completed","item":{"type":"agent_message","text":"Hello world"}}"#;
        let parsed = adapter.parse_output_line(line);
        assert_eq!(parsed.content, "Hello world");
        assert_eq!(parsed.line_type, super::LineType::Json);
        assert!(parsed.is_assistant);
    }

    #[test]
    fn parse_output_line_skips_control_events() {
        let adapter = CodexAdapter::new();
        // All known control/lifecycle events should be skipped
        for event in [
            "thread.started", "turn.started", "turn.completed", "item.delta",
            "item.started", "item.updated", "session.started", "session.completed"
        ] {
            let line = format!(r#"{{"type":"{}"}}"#, event);
            let parsed = adapter.parse_output_line(&line);
            assert!(parsed.content.is_empty(), "should skip {}", event);
            assert!(!parsed.is_assistant, "should not mark {} as assistant", event);
        }
    }

    #[test]
    fn parse_output_line_skips_error_events() {
        let adapter = CodexAdapter::new();
        // turn.failed and error events should be skipped (not pollute output)
        let turn_failed = r#"{"type":"turn.failed","error":{"message":"Rate limit exceeded"}}"#;
        let parsed = adapter.parse_output_line(turn_failed);
        assert!(parsed.content.is_empty(), "turn.failed should be skipped");
        assert!(!parsed.is_assistant);

        let error_event = r#"{"type":"error","error":{"message":"Connection failed"}}"#;
        let parsed = adapter.parse_output_line(error_event);
        assert!(parsed.content.is_empty(), "error should be skipped");
        assert!(!parsed.is_assistant);
    }

    #[test]
    fn parse_output_line_skips_unknown_typed_events() {
        let adapter = CodexAdapter::new();
        // Unknown events WITH a type field should be skipped
        let line = r#"{"type":"future.new.event","data":"something"}"#;
        let parsed = adapter.parse_output_line(line);
        assert!(parsed.content.is_empty(), "unknown typed event should be skipped");
        assert!(!parsed.is_assistant);
    }

    #[test]
    fn parse_output_line_passes_through_typeless_json() {
        let adapter = CodexAdapter::new();
        // JSON without "type" field (like mock data or Loop mode) should pass through
        let line = r#"{"question":"Hi","options":[]}"#;
        let parsed = adapter.parse_output_line(line);
        assert_eq!(parsed.content, line);
        assert_eq!(parsed.line_type, super::LineType::Text);
        assert!(parsed.is_assistant);
    }

    #[test]
    fn parse_output_line_handles_plain_text() {
        let adapter = CodexAdapter::new();
        let line = "Hello world";
        let parsed = adapter.parse_output_line(line);
        assert_eq!(parsed.content, "Hello world");
        assert_eq!(parsed.line_type, super::LineType::Text);
    }

    #[test]
    fn parse_output_line_preserves_completion_signal() {
        let adapter = CodexAdapter::new();
        let line = "<done>COMPLETE</done>";
        let parsed = adapter.parse_output_line(line);
        assert_eq!(parsed.content, "<done>COMPLETE</done>");
        assert!(parsed.is_assistant);
    }
}
