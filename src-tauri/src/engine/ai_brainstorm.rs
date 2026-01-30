use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// AI brainstorm response with structured options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiBrainstormResponse {
    /// The question text
    pub question: String,
    /// Optional description
    pub description: Option<String>,
    /// Available options (empty for text input)
    pub options: Vec<QuestionOption>,
    /// Whether multiple options can be selected
    pub multi_select: bool,
    /// Whether to show "Other" option for custom input
    pub allow_other: bool,
    /// Whether brainstorming is complete
    pub is_complete: bool,
    /// The generated prompt (only when is_complete is true)
    pub generated_prompt: Option<String>,
}

/// Question option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionOption {
    pub label: String,
    pub description: Option<String>,
    pub value: String,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

const BRAINSTORM_SYSTEM_PROMPT: &str = r#"你是一个帮助用户明确编程任务需求的助手。你的目标是通过对话了解用户想要完成的任务，收集足够的信息后生成一个完整的任务 prompt。

## 你的工作方式

1. 首先理解用户的初始描述
2. 提出问题来澄清需求，每次只问一个问题
3. 根据用户回答，决定是否需要继续追问
4. 当你认为已经收集到足够信息时，生成最终的任务 prompt

## 输出格式

你必须严格按照以下 JSON 格式输出，不要输出其他内容：

如果需要提问（带选项）：
```json
{
  "question": "你的问题",
  "description": "可选的问题描述",
  "options": [
    {"label": "选项标题", "description": "选项说明", "value": "选项值"},
    {"label": "选项标题2", "description": "选项说明2", "value": "选项值2"}
  ],
  "multiSelect": false,
  "allowOther": true,
  "isComplete": false
}
```

如果需要用户输入文本（不带选项）：
```json
{
  "question": "你的问题",
  "description": "可选的描述",
  "options": [],
  "multiSelect": false,
  "allowOther": false,
  "isComplete": false
}
```

如果已经收集够信息，准备生成 prompt：
```json
{
  "question": "需求收集完成",
  "description": "我已经了解了你的需求",
  "options": [],
  "multiSelect": false,
  "allowOther": false,
  "isComplete": true,
  "generatedPrompt": "完整的任务 prompt，包括任务描述、技术要求、具体功能列表、完成标准，最后加上完成信号：<done>COMPLETE</done>"
}
```

## 常见问题类型

1. 任务类型：新项目/添加功能/重构/修复bug
2. 技术栈选择
3. 具体功能需求
4. 测试要求
5. 其他约束

请用简洁友好的中文与用户对话。"#;

/// Run AI brainstorm with Claude Code
pub async fn run_ai_brainstorm(
    working_dir: &Path,
    conversation: &[ConversationMessage],
) -> Result<AiBrainstormResponse, String> {
    // Build the conversation context
    let mut context = String::new();

    for msg in conversation {
        if msg.role == "user" {
            context.push_str(&format!("用户: {}\n\n", msg.content));
        } else {
            context.push_str(&format!("助手: {}\n\n", msg.content));
        }
    }

    // Create the prompt for Claude
    let prompt = format!(
        "{}\n\n## 当前对话\n\n{}\n\n请根据对话历史，输出下一个问题的 JSON（或完成的 prompt）。只输出 JSON，不要其他内容。",
        BRAINSTORM_SYSTEM_PROMPT,
        context
    );

    // Call Claude Code CLI
    let output = call_claude_cli(working_dir, &prompt).await?;

    // Parse JSON response
    parse_ai_response(&output)
}

/// Parse AI response JSON
fn parse_ai_response(output: &str) -> Result<AiBrainstormResponse, String> {
    // Try to extract JSON from the output
    let json_str = extract_json(output)?;

    // Parse the JSON
    serde_json::from_str::<AiBrainstormResponse>(&json_str)
        .map_err(|e| format!("Failed to parse AI response: {}. Raw: {}", e, json_str))
}

/// Extract JSON from output (handles markdown code blocks)
fn extract_json(output: &str) -> Result<String, String> {
    let trimmed = output.trim();

    // Try to find JSON in code block
    if let Some(start) = trimmed.find("```json") {
        let json_start = start + 7;
        if let Some(end) = trimmed[json_start..].find("```") {
            return Ok(trimmed[json_start..json_start + end].trim().to_string());
        }
    }

    // Try to find JSON in generic code block
    if let Some(start) = trimmed.find("```") {
        let block_start = start + 3;
        // Skip language identifier if present
        let json_start = if let Some(newline) = trimmed[block_start..].find('\n') {
            block_start + newline + 1
        } else {
            block_start
        };
        if let Some(end) = trimmed[json_start..].find("```") {
            return Ok(trimmed[json_start..json_start + end].trim().to_string());
        }
    }

    // Try to find raw JSON object
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return Ok(trimmed[start..=end].to_string());
        }
    }

    Err(format!("No JSON found in output: {}", output))
}

/// Call Claude Code CLI and get response
async fn call_claude_cli(working_dir: &Path, prompt: &str) -> Result<String, String> {
    let mut cmd = Command::new("claude");
    cmd.arg("--print")
        .arg("--dangerously-skip-permissions")
        .arg("-p")
        .arg(prompt)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn claude: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut output = String::new();

    while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
        output.push_str(&line);
        output.push('\n');
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(format!("Claude CLI exited with status: {}", status));
    }

    Ok(output)
}
