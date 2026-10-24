use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::get_claude_config_dir;
use crate::session_manager::{SessionMessage, SessionMeta};

use super::utils::{
    extract_text, parse_timestamp_to_ms, path_basename, read_head_tail_lines, truncate_summary,
    TITLE_MAX_CHARS,
};

const PROVIDER_ID: &str = "claude";

pub fn scan_sessions() -> Vec<SessionMeta> {
    let root = get_claude_config_dir().join("projects");
    let mut files = Vec::new();
    collect_jsonl_files(&root, &mut files);

    let mut sessions = Vec::new();
    for path in files {
        if let Some(meta) = parse_session(&path) {
            sessions.push(meta);
        }
    }

    sessions
}

pub fn load_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open session file: {e}"))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(value) => value,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&line) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        if value.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }

        let message = match value.get("message") {
            Some(message) => message,
            None => continue,
        };

        let mut role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        // Claude wraps tool_result inside user messages; reclassify as "tool" role
        if role == "user" {
            if let Some(Value::Array(items)) = message.get("content") {
                let all_tool_results = !items.is_empty()
                    && items.iter().all(|item| {
                        item.get("type").and_then(Value::as_str) == Some("tool_result")
                    });
                if all_tool_results {
                    role = "tool".to_string();
                }
            }
        }

        let content = message.get("content").map(extract_text).unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }

        let ts = value.get("timestamp").and_then(parse_timestamp_to_ms);

        messages.push(SessionMessage { role, content, ts });
    }

    Ok(messages)
}

pub fn delete_session(_root: &Path, path: &Path, session_id: &str) -> Result<bool, String> {
    let meta = parse_session(path).ok_or_else(|| {
        format!(
            "Failed to parse Claude session metadata: {}",
            path.display()
        )
    })?;

    if meta.session_id != session_id {
        return Err(format!(
            "Claude session ID mismatch: expected {session_id}, found {}",
            meta.session_id
        ));
    }

    if let Some(stem) = path.file_stem() {
        let sibling = path.parent().unwrap_or_else(|| Path::new("")).join(stem);
        remove_path_if_exists(&sibling).map_err(|e| {
            format!(
                "Failed to delete Claude session sidecar {}: {e}",
                sibling.display()
            )
        })?;
    }

    std::fs::remove_file(path).map_err(|e| {
        format!(
            "Failed to delete Claude session file {}: {e}",
            path.display()
        )
    })?;

    Ok(true)
}

fn parse_session(path: &Path) -> Option<SessionMeta> {
    scan_session_file(path).ok().flatten()
}

/// Claude Code keeps session metadata (`last-prompt`, `custom-title`, `relocated`, …)
/// within the last 64 KiB of the transcript instead of appending it after every turn,
/// so a `/cd` record can sit tens of kilobytes before EOF. The 30-line head/tail
/// window used for titles is not enough to find it.
const RELOCATED_TAIL_BYTES: u64 = 64 * 1024;

/// Last `/cd` target recorded in the transcript, read from the same 64 KiB window
/// Claude Code itself uses. `None` when the transcript has no relocated record.
fn last_relocated_cwd(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let start = len.saturating_sub(RELOCATED_TAIL_BYTES);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<&str> = text.lines().collect();
    // Seeking into the middle of a line leaves a partial first line.
    if start > 0 && !lines.is_empty() {
        lines.remove(0);
    }
    lines
        .into_iter()
        .rev()
        .filter(|line| line.contains("\"relocated\""))
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("type").and_then(Value::as_str) == Some("relocated"))
        .find_map(|value| {
            value
                .get("relocatedCwd")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|cwd| !cwd.is_empty())
                .map(str::to_string)
        })
}

/// 列表扫描用：读不了（权限、文件被占用等）返回 `Err`，不进解析缓存、下轮重试；
/// 读到了但不是会话返回 `Ok(None)`，可以缓存。
fn scan_session_file(path: &Path) -> std::io::Result<Option<SessionMeta>> {
    if is_agent_session(path) {
        return Ok(None);
    }
    let (head, tail) = read_head_tail_lines(path, 10, 30)?;
    let relocated_cwd = last_relocated_cwd(path);
    Ok(parse_session_lines(path, head, tail, relocated_cwd))
}

fn parse_session_lines(
    path: &Path,
    head: Vec<String>,
    tail: Vec<String>,
    relocated_cwd: Option<String>,
) -> Option<SessionMeta> {
    let mut session_id: Option<String> = None;
    let mut project_dir: Option<String> = None;
    let mut created_at: Option<i64> = None;
    let mut first_user_message: Option<String> = None;

    // Extract metadata and first user message from head lines
    for line in &head {
        let value: Value = match serde_json::from_str(line) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };
        if session_id.is_none() {
            session_id = value
                .get("sessionId")
                .and_then(Value::as_str)
                .map(|s| s.to_string());
        }
        if project_dir.is_none() {
            project_dir = value
                .get("cwd")
                .and_then(Value::as_str)
                .map(|s| s.to_string());
        }
        if created_at.is_none() {
            created_at = value.get("timestamp").and_then(parse_timestamp_to_ms);
        }
        // Extract first real user message as title candidate
        // Skip system-injected caveats and slash commands (e.g. /clear, /compact)
        if first_user_message.is_none() {
            let is_user = value.get("type").and_then(Value::as_str) == Some("user")
                || value
                    .get("message")
                    .and_then(|m| m.get("role"))
                    .and_then(Value::as_str)
                    == Some("user");
            if is_user {
                if let Some(message) = value.get("message") {
                    let text = message.get("content").map(extract_text).unwrap_or_default();
                    let trimmed = text.trim();
                    if !trimmed.is_empty()
                        && !trimmed.starts_with("<command-name>")
                        && !is_injected_text(trimmed)
                    {
                        first_user_message = Some(trimmed.to_string());
                    }
                }
            }
        }
        if session_id.is_some()
            && project_dir.is_some()
            && created_at.is_some()
            && first_user_message.is_some()
        {
            break;
        }
    }

    // Extract last_active_at, summary, and custom-title from tail lines (reverse order).
    let mut last_active_at: Option<i64> = None;
    let mut summary: Option<String> = None;
    let mut custom_title: Option<String> = None;

    for line in tail.iter().rev() {
        let value: Value = match serde_json::from_str(line) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };
        if last_active_at.is_none() {
            last_active_at = value.get("timestamp").and_then(parse_timestamp_to_ms);
        }
        // Look for custom-title entry (take the last one, i.e. first in reverse)
        if custom_title.is_none()
            && value.get("type").and_then(Value::as_str) == Some("custom-title")
        {
            custom_title = value
                .get("customTitle")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
        if summary.is_none() {
            if value.get("isMeta").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            if let Some(message) = value.get("message") {
                let text = message.get("content").map(extract_text).unwrap_or_default();
                let trimmed = text.trim();
                // A local command run last, such as `/model`, ends the file with its command and output.
                if !trimmed.is_empty()
                    && !trimmed.starts_with("<command-name>")
                    && !is_injected_text(trimmed)
                {
                    summary = Some(text);
                }
            }
        }
        if last_active_at.is_some() && summary.is_some() && custom_title.is_some() {
            break;
        }
    }

    // `/cd` moved the session; the last relocated record wins over the cwd the
    // transcript started with. Sessions without a relocated record keep that cwd.
    //
    // Ordinary records are deliberately ignored: their `cwd` is the Bash tool's
    // tracked working directory and drifts into subdirectories as commands run,
    // which must not move the session.
    if relocated_cwd.is_some() {
        project_dir = relocated_cwd;
    }

    let session_id = session_id.or_else(|| infer_session_id_from_filename(path));
    let session_id = session_id?;

    // Title priority: custom-title > first user message > directory basename
    let title = custom_title
        .map(|t| truncate_summary(&t, TITLE_MAX_CHARS))
        .or_else(|| first_user_message.map(|t| truncate_summary(&t, TITLE_MAX_CHARS)))
        .or_else(|| {
            project_dir
                .as_deref()
                .and_then(path_basename)
                .map(|v| v.to_string())
        });

    let summary = summary.map(|text| truncate_summary(&text, 160));

    Some(SessionMeta {
        provider_id: PROVIDER_ID.to_string(),
        session_id: session_id.clone(),
        title,
        summary,
        project_dir,
        created_at,
        last_active_at,
        source_path: Some(path.to_string_lossy().to_string()),
        resume_command: Some(format!("claude --resume {session_id}")),
    })
}

fn is_agent_session(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("agent-"))
        .unwrap_or(false)
}

fn infer_session_id_from_filename(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_string())
}

fn collect_jsonl_files(root: &Path, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
}

fn remove_path_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::metadata(path) {
        Ok(meta) => {
            if meta.is_dir() {
                std::fs::remove_dir_all(path)
            } else {
                std::fs::remove_file(path)
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn delete_session_removes_main_file_and_sidecar_directory() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("abc123-session.jsonl");
        let sidecar = temp.path().join("abc123-session");
        let subagents = sidecar.join("subagents");
        let tool_results = sidecar.join("tool-results");

        std::fs::create_dir_all(&subagents).expect("create subagents");
        std::fs::create_dir_all(&tool_results).expect("create tool-results");
        std::fs::write(subagents.join("agent-1.jsonl"), "{}").expect("write subagent");
        std::fs::write(tool_results.join("tool-1.txt"), "result").expect("write tool result");
        std::fs::write(
            &path,
            concat!(
                "{\"sessionId\":\"session-123\",\"cwd\":\"/tmp/project\",\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
                "{\"message\":{\"role\":\"user\",\"content\":\"hello\"},\"timestamp\":\"2026-03-06T10:01:00Z\"}\n"
            ),
        )
        .expect("write session");

        delete_session(temp.path(), &path, "session-123").expect("delete session");

        assert!(!path.exists());
        assert!(!sidecar.exists());
    }

    #[test]
    fn load_messages_tool_use_shows_as_assistant() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"Write\",\"input\":{\"file_path\":\"a.txt\"}}]},\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
                "{\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"tool_result\",\"tool_use_id\":\"toolu_1\",\"content\":\"File written\"}]},\"timestamp\":\"2026-03-06T10:00:01Z\"}\n",
            ),
        )
        .expect("write");

        let msgs = load_messages(&path).expect("load");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "assistant");
        assert!(msgs[0].content.contains("[Tool: Write]"));
        assert_eq!(msgs[1].role, "tool");
        assert_eq!(msgs[1].content, "File written");
    }

    #[test]
    fn load_messages_mixed_text_and_tool_use() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");
        std::fs::write(
            &path,
            "{\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"Let me help.\"},{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"Read\",\"input\":{}}]},\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
        )
        .expect("write");

        let msgs = load_messages(&path).expect("load");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "assistant");
        assert!(msgs[0].content.contains("Let me help."));
        assert!(msgs[0].content.contains("[Tool: Read]"));
    }

    #[test]
    fn load_messages_mixed_user_tool_result_and_text_stays_user() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");
        std::fs::write(
            &path,
            "{\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"tool_result\",\"tool_use_id\":\"toolu_1\",\"content\":\"result\"},{\"type\":\"text\",\"text\":\"Please continue\"}]},\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
        )
        .expect("write");

        let msgs = load_messages(&path).expect("load");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "user");
        assert!(msgs[0].content.contains("Please continue"));
    }

    #[test]
    fn parse_session_uses_first_user_message_as_title() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-abc.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"sessionId\":\"session-abc\",\"cwd\":\"/tmp/project\",\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"How do I deploy?\"},\"sessionId\":\"session-abc\",\"timestamp\":\"2026-03-06T10:01:00Z\"}\n",
                "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"Here is how...\"},\"timestamp\":\"2026-03-06T10:02:00Z\"}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        assert_eq!(meta.title.as_deref(), Some("How do I deploy?"));
    }

    #[test]
    fn parse_session_custom_title_overrides_first_message() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-def.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"sessionId\":\"session-def\",\"cwd\":\"/tmp/project\",\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"fix something\"},\"sessionId\":\"session-def\",\"timestamp\":\"2026-03-06T10:01:00Z\"}\n",
                "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"Done.\"},\"timestamp\":\"2026-03-06T10:02:00Z\"}\n",
                "{\"type\":\"custom-title\",\"customTitle\":\"fix-login-bug\",\"sessionId\":\"session-def\"}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        assert_eq!(meta.title.as_deref(), Some("fix-login-bug"));
    }

    #[test]
    fn parse_session_prefers_last_relocated_cwd_after_cd() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-cd.jsonl");
        let mut lines = vec![
            r#"{"type":"user","sessionId":"session-cd","cwd":"/tmp/project-a","timestamp":"2026-03-06T10:00:00Z","message":{"role":"user","content":"start"}}"#
                .to_string(),
        ];
        // Pad past the 16 KB boundary so the head/tail reader takes its seek branch.
        for i in 1..=160 {
            lines.push(format!(
                r#"{{"type":"assistant","cwd":"/tmp/project-a","timestamp":"2026-03-06T10:{:02}:00Z","message":{{"role":"assistant","content":"step {i} {}"}}}}"#,
                i % 60,
                "x".repeat(100)
            ));
        }
        lines.push(
            r#"{"type":"relocated","relocatedCwd":"/tmp/project-b","sessionId":"session-cd","timestamp":"2026-03-06T11:00:00Z"}"#
                .to_string(),
        );
        lines.push(
            r#"{"type":"relocated","relocatedCwd":"/tmp/project-c","sessionId":"session-cd","timestamp":"2026-03-06T11:05:00Z"}"#
                .to_string(),
        );
        for i in 0..3 {
            lines.push(format!(
                r#"{{"type":"assistant","cwd":"/tmp/project-c/subdir","timestamp":"2026-03-06T11:{:02}:00Z","message":{{"role":"assistant","content":"after cd {i}"}}}}"#,
                10 + i
            ));
        }
        std::fs::write(&path, format!("{}\n", lines.join("\n"))).expect("write");
        assert!(std::fs::metadata(&path).expect("metadata").len() > 16_384);

        let meta = parse_session(&path).unwrap();
        // The last `relocated` record wins, and the drifting `cwd` of later
        // records (the Bash tool's tracked directory) is ignored.
        assert_eq!(meta.project_dir.as_deref(), Some("/tmp/project-c"));
    }

    #[test]
    fn parse_session_finds_relocated_cwd_beyond_the_tail_window() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-cd-far.jsonl");
        let mut lines = vec![
            r#"{"type":"user","sessionId":"session-cd-far","cwd":"/tmp/project-a","timestamp":"2026-03-06T10:00:00Z","message":{"role":"user","content":"start"}}"#
                .to_string(),
            r#"{"type":"relocated","relocatedCwd":"/tmp/project-worktree","sessionId":"session-cd-far","timestamp":"2026-03-06T11:00:00Z"}"#
                .to_string(),
        ];
        // The metadata block is not rewritten after every turn, so push the last
        // relocated record past both the 16 KiB and the 30-line tail window.
        for i in 0..120 {
            lines.push(format!(
                r#"{{"type":"assistant","cwd":"/tmp/project-worktree","timestamp":"2026-03-06T11:{:02}:00Z","message":{{"role":"assistant","content":"after cd {i} {}"}}}}"#,
                i % 60,
                "y".repeat(240)
            ));
        }
        std::fs::write(&path, format!("{}\n", lines.join("\n"))).expect("write");

        let (_, tail) = read_head_tail_lines(&path, 10, 30).expect("head/tail");
        assert!(
            !tail.iter().any(|line| line.contains("\"relocated\"")),
            "test fixture must place the relocated record outside the 30-line tail"
        );

        let meta = parse_session(&path).unwrap();
        assert_eq!(meta.project_dir.as_deref(), Some("/tmp/project-worktree"));
    }

    #[test]
    fn parse_session_ignores_drifting_cwd_without_relocated_record() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-cwd-drift.jsonl");
        let mut lines = vec![
            r#"{"type":"user","sessionId":"session-cwd-drift","cwd":"/tmp/project-a","timestamp":"2026-03-06T10:00:00Z","message":{"role":"user","content":"start"}}"#
                .to_string(),
        ];
        for i in 0..40 {
            lines.push(format!(
                r#"{{"type":"assistant","cwd":"/tmp/project-a/subdir","timestamp":"2026-03-06T10:{:02}:00Z","message":{{"role":"assistant","content":"drifted {i}"}}}}"#,
                i % 60
            ));
        }
        std::fs::write(&path, format!("{}\n", lines.join("\n"))).expect("write");

        let meta = parse_session(&path).unwrap();
        // A Bash `cd` into a subdirectory must not move the session's project.
        assert_eq!(meta.project_dir.as_deref(), Some("/tmp/project-a"));
    }

    #[test]
    fn parse_session_falls_back_to_dir_basename() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-ghi.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"sessionId\":\"session-ghi\",\"cwd\":\"/tmp/my-project\",\"timestamp\":\"2026-03-06T10:00:00Z\"}\n",
                "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"timestamp\":\"2026-03-06T10:01:00Z\"}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        // No user message and no custom-title → falls back to dir basename
        assert_eq!(meta.title.as_deref(), Some("my-project"));
    }

    #[test]
    fn parse_session_truncates_long_title() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-trunc.jsonl");
        let long_msg = "a".repeat(200);
        std::fs::write(
            &path,
            format!(
                "{{\"sessionId\":\"session-trunc\",\"cwd\":\"/tmp/p\",\"timestamp\":\"2026-03-06T10:00:00Z\"}}\n\
                 {{\"type\":\"user\",\"message\":{{\"role\":\"user\",\"content\":\"{long_msg}\"}},\"sessionId\":\"session-trunc\",\"timestamp\":\"2026-03-06T10:01:00Z\"}}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        let title = meta.title.unwrap();
        assert!(title.len() <= TITLE_MAX_CHARS + 3); // +3 for "..."
        assert!(title.ends_with("..."));
    }

    #[test]
    fn parse_session_new_format_with_snapshot() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-new.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"type\":\"file-history-snapshot\",\"messageId\":\"msg-1\",\"snapshot\":{},\"isSnapshotUpdate\":false}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"请帮我重构这个函数\"},\"sessionId\":\"session-new\",\"timestamp\":\"2026-03-06T10:00:00Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"OK\"},\"timestamp\":\"2026-03-06T10:01:00Z\",\"cwd\":\"/tmp/project\"}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        assert_eq!(meta.title.as_deref(), Some("请帮我重构这个函数"));
    }

    #[test]
    fn parse_session_skips_command_caveat_and_slash_commands() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-clear.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"type\":\"file-history-snapshot\",\"messageId\":\"msg-1\",\"snapshot\":{},\"isSnapshotUpdate\":false}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"<local-command-caveat>Caveat: The messages below were generated by the user while running local commands.</local-command-caveat>\"},\"sessionId\":\"session-clear\",\"timestamp\":\"2026-03-06T10:00:00Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"<command-name>/clear</command-name>\\n<command-message>clear</command-message>\"},\"sessionId\":\"session-clear\",\"timestamp\":\"2026-03-06T10:00:01Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"Done.\"},\"timestamp\":\"2026-03-06T10:00:02Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"帮我看看工作区的改动\"},\"sessionId\":\"session-clear\",\"timestamp\":\"2026-03-06T10:01:00Z\",\"cwd\":\"/tmp/project\"}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        assert_eq!(meta.title.as_deref(), Some("帮我看看工作区的改动"));
    }

    #[test]
    fn parse_session_skips_local_command_output_in_title_and_summary() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session-model.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"type\":\"user\",\"isMeta\":true,\"message\":{\"role\":\"user\",\"content\":\"<local-command-caveat>Caveat: The messages below were generated by the user while running local commands.</local-command-caveat>\"},\"sessionId\":\"session-model\",\"timestamp\":\"2026-03-06T10:00:00Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"<command-name>/model</command-name>\\n<command-message>model</command-message>\\n<command-args>opus</command-args>\"},\"sessionId\":\"session-model\",\"timestamp\":\"2026-03-06T10:00:01Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"<local-command-stdout>Set model to opus</local-command-stdout>\"},\"sessionId\":\"session-model\",\"timestamp\":\"2026-03-06T10:00:02Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"Fix the login redirect\"},\"sessionId\":\"session-model\",\"timestamp\":\"2026-03-06T10:01:00Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"The redirect now keeps the return URL.\"},\"timestamp\":\"2026-03-06T10:02:00Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"<command-name>/model</command-name>\\n<command-message>model</command-message>\\n<command-args>sonnet</command-args>\"},\"sessionId\":\"session-model\",\"timestamp\":\"2026-03-06T10:03:00Z\",\"cwd\":\"/tmp/project\"}\n",
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"<local-command-stdout>Set model to sonnet</local-command-stdout>\"},\"sessionId\":\"session-model\",\"timestamp\":\"2026-03-06T10:03:01Z\",\"cwd\":\"/tmp/project\"}\n",
            ),
        )
        .expect("write");

        let meta = parse_session(&path).unwrap();
        assert_eq!(meta.title.as_deref(), Some("Fix the login redirect"));
        assert_eq!(
            meta.summary.as_deref(),
            Some("The redirect now keeps the return URL.")
        );
    }

    #[test]
    fn workflow_journal_log_is_excluded_from_sessions() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("journal.jsonl");
        std::fs::write(
            &path,
            "{\"type\":\"started\",\"key\":\"k\",\"agentId\":\"a\"}\n",
        )
        .expect("write");

        assert!(is_agent_session(&path));
    }

    #[test]
    fn agent_session_files_are_still_excluded() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("agent-abc123.jsonl");
        std::fs::write(&path, "{\"type\":\"user\",\"isSidechain\":true}\n").expect("write");

        assert!(is_agent_session(&path));
    }

    #[test]
    fn real_session_file_is_not_excluded() {
        let temp = tempdir().expect("tempdir");
        let path = temp
            .path()
            .join("8f8e7c8e-0000-0000-0000-000000000000.jsonl");
        std::fs::write(
            &path,
            "{\"sessionId\":\"8f8e7c8e-0000-0000-0000-000000000000\"}\n",
        )
        .expect("write");

        assert!(!is_agent_session(&path));
    }
}

#[cfg(test)]
mod transcript_tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    /// 把记录写成 JSONL 并解析；返回消息与每行的字节偏移（用来校验 ContentRef）。
    fn parse_lines(lines: &[Value]) -> (tempfile::TempDir, PathBuf, Vec<SessionMessage>, Vec<u64>) {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");
        let mut body = String::new();
        let mut offsets = Vec::new();
        for line in lines {
            offsets.push(body.len() as u64);
            body.push_str(&serde_json::to_string(line).unwrap());
            body.push('\n');
        }
        std::fs::write(&path, body).expect("write");
        let messages = load_messages(&path).expect("load");
        (temp, path, messages, offsets)
    }

    fn user(uuid: &str, content: Value) -> Value {
        json!({
            "type": "user", "uuid": uuid, "timestamp": "2026-10-03T10:00:00Z",
            "message": { "role": "user", "content": content }
        })
    }

    fn assistant(uuid: &str, msg_id: &str, content: Value) -> Value {
        json!({
            "type": "assistant", "uuid": uuid, "timestamp": "2026-10-03T10:00:01Z",
            "message": { "id": msg_id, "role": "assistant", "model": "claude-opus-5-5", "content": content }
        })
    }

    fn block_types(message: &SessionMessage) -> Vec<String> {
        serde_json::to_value(&message.blocks)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["type"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn thinking_redacted_and_split_records_merge() {
        let (_t, _p, msgs, _) = parse_lines(&[
            user("u1", json!("hi")),
            {
                let mut v = assistant(
                    "a1",
                    "msg_1",
                    json!([{ "type": "thinking", "thinking": "", "signature": "EuYBCkQ" }]),
                );
                v["thinkingDurationMs"] = json!(3200);
                v
            },
            assistant("a2", "msg_1", json!([{ "type": "text", "text": "answer" }])),
            assistant(
                "a3",
                "msg_2",
                json!([{ "type": "thinking", "thinking": "x".repeat(500), "signature": "s" }]),
            ),
            // 既无正文又无签名的 thinking 直接丢弃
            assistant(
                "a4",
                "msg_3",
                json!([{ "type": "thinking", "thinking": "" }]),
            ),
        ]);
        assert_eq!(msgs.len(), 3, "同 message.id 的拆分记录合并为一条");
        assert_eq!(msgs[1].id.as_deref(), Some("a1"));
        assert_eq!(block_types(&msgs[1]), ["thinking", "text"]);
        assert_eq!(msgs[1].content, "answer", "thinking 不进入 content");
        match &msgs[1].blocks[0] {
            SessionBlock::Thinking {
                text,
                redacted,
                duration_ms,
                full,
                ..
            } => {
                assert!(text.is_empty());
                assert!(*redacted);
                assert_eq!(*duration_ms, Some(3200));
                assert!(full.is_none());
            }
            other => panic!("unexpected {other:?}"),
        }
        match &msgs[2].blocks[0] {
            SessionBlock::Thinking {
                text,
                redacted,
                full,
                ..
            } => {
                assert_eq!(text.chars().count(), THINKING_PREVIEW_CHARS);
                assert!(!*redacted);
                assert!(matches!(
                    full,
                    Some(ContentRef::Jsonl { pointer, .. }) if pointer == "/message/content/0/thinking"
                ));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn tool_result_images_become_refs() {
        let data = "QUJD".repeat(1000);
        let (_t, path, msgs, offsets) = parse_lines(&[
            user("u1", json!("截个图")),
            assistant(
                "a1",
                "msg_1",
                json!([{ "type": "tool_use", "id": "toolu_1", "name": "mcp__claude-in-chrome__computer", "input": { "action": "screenshot", "tabId": 1 } }]),
            ),
            user(
                "u2",
                json!([{ "type": "tool_result", "tool_use_id": "toolu_1", "content": [
                    { "type": "text", "text": "Successfully captured screenshot" },
                    { "type": "image", "source": { "type": "base64", "media_type": "image/jpeg", "data": data } }
                ]}]),
            ),
            user(
                "u3",
                json!([
                    { "type": "text", "text": "看这张图" },
                    { "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": data } }
                ]),
            ),
        ]);
        let raw = std::fs::read(&path).unwrap();

        assert_eq!(
            msgs[1].content,
            "[Tool: mcp__claude-in-chrome__computer] claude-in-chrome.computer"
        );
        match &msgs[1].blocks[0] {
            SessionBlock::ToolCall {
                kind,
                server,
                detail,
                ..
            } => {
                assert_eq!(*kind, ToolKind::Mcp);
                assert_eq!(server.as_deref(), Some("claude-in-chrome"));
                assert_eq!(detail.as_deref(), Some("screenshot"));
            }
            other => panic!("unexpected {other:?}"),
        }

        assert_eq!(msgs[2].role, "tool");
        let SessionBlock::ToolResult {
            images, preview, ..
        } = &msgs[2].blocks[0]
        else {
            panic!("expected tool_result");
        };
        assert_eq!(preview, "Successfully captured screenshot");
        assert_eq!(images.len(), 1);
        let image = &images[0];
        assert_eq!(image.media_type, "image/jpeg");
        assert_eq!(image.size, 3000);
        let ImageSource::Inline {
            content:
                ContentRef::Jsonl {
                    offset,
                    len,
                    pointer,
                },
        } = &image.source
        else {
            panic!("expected inline jsonl ref");
        };
        assert_eq!(*offset, offsets[2]);
        assert_eq!(pointer, "/message/content/0/content/1/source/data");
        // 引用能回取：行区间可解析，pointer 指向 base64 原文
        let line: Value =
            serde_json::from_slice(&raw[*offset as usize..*offset as usize + *len as usize])
                .unwrap();
        assert_eq!(
            line.pointer(pointer).and_then(Value::as_str),
            Some(data.as_str())
        );
        assert!(
            !serde_json::to_string(&msgs).unwrap().contains(&data),
            "不内联 base64"
        );

        // 用户贴图
        assert_eq!(msgs[3].role, "user");
        assert_eq!(block_types(&msgs[3]), ["text", "image"]);
        assert_eq!(msgs[3].content, "看这张图\n\n[Image: image/png 3000]");
        assert_eq!(msgs[3].turn_id.as_deref(), Some("t2"));
    }

    #[test]
    fn edit_diff_is_backfilled_from_structured_patch() {
        let mut result = user(
            "u2",
            json!([{ "type": "tool_result", "tool_use_id": "toolu_e", "content": "The file a.rs has been updated successfully." }]),
        );
        result["toolUseResult"] = json!({
            "filePath": "/p/a.rs",
            "oldString": "x",
            "newString": "y",
            "structuredPatch": [
                { "oldStart": 1, "oldLines": 3, "newStart": 1, "newLines": 4, "lines": [" a", "-b", "+c", "+d", " e"] },
                { "oldStart": 9, "oldLines": 1, "newStart": 10, "newLines": 1, "lines": ["-f", "+g"] }
            ]
        });
        let mut create = user(
            "u4",
            json!([{ "type": "tool_result", "tool_use_id": "toolu_w", "content": "File created successfully at: /p/new.md" }]),
        );
        create["toolUseResult"] = json!({ "type": "create", "filePath": "/p/new.md", "content": "1\n2\n3\n", "structuredPatch": [] });
        let failed = user(
            "u6",
            json!([{ "type": "tool_result", "tool_use_id": "toolu_f", "is_error": true, "content": "String to replace not found" }]),
        );

        let (_t, _p, msgs, _) = parse_lines(&[
            user("u1", json!("改一下")),
            assistant(
                "a1",
                "m1",
                json!([{ "type": "tool_use", "id": "toolu_e", "name": "Edit", "input": { "file_path": "/p/a.rs", "old_string": "b", "new_string": "c\nd" } }]),
            ),
            result,
            assistant(
                "a2",
                "m2",
                json!([{ "type": "tool_use", "id": "toolu_w", "name": "Write", "input": { "file_path": "/p/new.md", "content": "1\n2\n3\n" } }]),
            ),
            create,
            assistant(
                "a3",
                "m3",
                json!([{ "type": "tool_use", "id": "toolu_f", "name": "Edit", "input": { "file_path": "/p/b.rs", "old_string": "q", "new_string": "r" } }]),
            ),
            failed,
        ]);

        let diff_of = |m: &SessionMessage| match &m.blocks[0] {
            SessionBlock::ToolCall { diff, .. } => diff.clone(),
            other => panic!("unexpected {other:?}"),
        };
        let edit = diff_of(&msgs[1]).expect("edit diff");
        assert_eq!(
            (edit.added, edit.removed),
            (3, 2),
            "以 structuredPatch 为准"
        );
        assert_eq!(edit.files[0].path, "/p/a.rs");
        assert_eq!(edit.files[0].op, DiffOp::Update);

        let write = diff_of(&msgs[3]).expect("write diff");
        assert_eq!(write.files[0].op, DiffOp::Add);
        assert_eq!((write.added, write.removed), (3, 0), "新建文件按内容行数");

        assert!(diff_of(&msgs[5]).is_none(), "失败的改动不显示 diff");
        let SessionBlock::ToolResult {
            status, exit_code, ..
        } = &msgs[6].blocks[0]
        else {
            panic!("expected tool_result");
        };
        assert_eq!(*status, ToolStatus::Error);
        assert_eq!(*exit_code, None);
    }

    #[test]
    fn system_reminders_are_stripped_or_marked_injected() {
        let (_t, _p, msgs, _) = parse_lines(&[
            user(
                "u1",
                json!("<system-reminder>\nThe user opened a.rs\n</system-reminder>\n\n修一下这个 bug"),
            ),
            user("u2", json!("<system-reminder>\nonly a reminder\n</system-reminder>")),
            user(
                "u3",
                json!("<task-notification>\n<task-id>t1</task-id>\n<status>completed</status>\n</task-notification>"),
            ),
            user(
                "u4",
                json!([
                    { "type": "tool_result", "tool_use_id": "toolu_x", "content": "ok\n<system-reminder>noise</system-reminder>" },
                    { "type": "text", "text": "<system-reminder>also noise</system-reminder>" }
                ]),
            ),
        ]);
        assert_eq!(msgs.len(), 4);
        assert_eq!(msgs[0].content, "修一下这个 bug");
        assert!(!msgs[0].injected);
        assert_eq!(msgs[0].turn_id.as_deref(), Some("t1"));

        assert!(msgs[1].injected);
        assert!(
            msgs[1].content.starts_with("<system-reminder>"),
            "整条注入时保留原文"
        );
        assert_eq!(msgs[1].turn_id.as_deref(), Some("t1"), "注入内容不开新轮");

        assert!(msgs[2].injected);
        assert_eq!(msgs[2].turn_id.as_deref(), Some("t1"));

        assert_eq!(msgs[3].role, "tool");
        assert_eq!(msgs[3].content, "ok");
        assert!(!msgs[3].injected);
    }

    #[test]
    fn slash_commands_interrupts_and_system_events() {
        let mut boundary = json!({ "type": "system", "subtype": "compact_boundary", "content": "Conversation compacted", "uuid": "s1", "timestamp": "2026-10-03T10:01:00Z" });
        boundary["compactMetadata"] = json!({ "trigger": "manual" });
        let (_t, _p, msgs, _) = parse_lines(&[
            user("u1", json!("第一问")),
            json!({ "type": "pr-link", "prNumber": 128, "prUrl": "https://github.com/o/r/pull/128", "timestamp": "2026-10-03T10:00:02Z" }),
            json!({ "type": "pr-link", "prNumber": 128, "prUrl": "https://github.com/o/r/pull/128", "timestamp": "2026-10-03T10:00:03Z" }),
            json!({ "type": "attachment", "attachment": { "type": "deferred_tools_delta" } }),
            json!({ "type": "user", "isMeta": true, "message": { "role": "user", "content": "<local-command-caveat>Caveat</local-command-caveat>" } }),
            user(
                "u2",
                json!("<command-name>/compact</command-name>\n<command-message>compact</command-message>\n<command-args></command-args>"),
            ),
            boundary,
            json!({ "type": "user", "isCompactSummary": true, "uuid": "u3", "message": { "role": "user", "content": "This session is being continued…" } }),
            user("u4", json!("<local-command-stdout>Compacted </local-command-stdout>")),
            user("u5", json!([{ "type": "text", "text": "[Request interrupted by user for tool use]" }])),
            json!({ "type": "system", "subtype": "stop_hook_summary", "hookErrors": [], "timestamp": "2026-10-03T10:02:00Z" }),
            json!({ "type": "system", "subtype": "stop_hook_summary", "hookErrors": ["notify.sh exited with code 127"], "timestamp": "2026-10-03T10:02:01Z" }),
            json!({ "type": "system", "subtype": "local_command", "uuid": "s2", "content": "<command-name>/status</command-name>\n<command-args>verbose</command-args>" }),
            user("u6", json!("<bash-input>git status</bash-input><bash-stdout>clean</bash-stdout><bash-stderr></bash-stderr>")),
        ]);

        let summary: Vec<(String, String, String, bool, String)> = msgs
            .iter()
            .map(|m| {
                (
                    m.role.clone(),
                    m.turn_id.clone().unwrap(),
                    block_types(m).join(","),
                    m.injected,
                    m.content.clone(),
                )
            })
            .collect();
        let row = |role: &str, turn: &str, types: &str, injected: bool, content: &str| {
            (
                role.to_string(),
                turn.to_string(),
                types.to_string(),
                injected,
                content.to_string(),
            )
        };
        assert_eq!(
            summary,
            vec![
                row("user", "t1", "text", false, "第一问"),
                row("system", "t1", "event", false, "PR #128"),
                row("user", "t2", "event", false, "/compact"),
                row(
                    "system",
                    "t2",
                    "event",
                    false,
                    "This session is being continued…"
                ),
                row(
                    "user",
                    "t2",
                    "text",
                    true,
                    "<local-command-stdout>Compacted </local-command-stdout>"
                ),
                row(
                    "user",
                    "t2",
                    "event",
                    false,
                    "[Request interrupted by user for tool use]"
                ),
                row(
                    "system",
                    "t2",
                    "event",
                    false,
                    "notify.sh exited with code 127"
                ),
                row("user", "t3", "event", false, "/status verbose"),
                row(
                    "user",
                    "t3",
                    "tool_call,tool_result",
                    false,
                    "[Tool: Bash] git status\n\nclean"
                ),
            ]
        );
        let kinds: Vec<EventKind> = msgs
            .iter()
            .filter_map(|m| match m.blocks.first() {
                Some(SessionBlock::Event { kind, .. }) => Some(*kind),
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            [
                EventKind::PrLink,
                EventKind::SlashCommand,
                EventKind::Compaction,
                EventKind::Aborted,
                EventKind::Hook,
                EventKind::SlashCommand
            ]
        );
        assert!(matches!(
            &msgs[1].blocks[0],
            SessionBlock::Event { url: Some(url), .. } if url == "https://github.com/o/r/pull/128"
        ));
        assert!(matches!(
            &msgs[8].blocks[0],
            SessionBlock::ToolCall { by_user: true, .. }
        ));
    }

    #[test]
    fn tool_titles_project_into_content() {
        let (_t, _p, msgs, _) = parse_lines(&[assistant(
            "a1",
            "m1",
            json!([
                { "type": "tool_use", "id": "t1", "name": "Bash", "input": { "command": "rg -n foo src\necho done", "description": "Search foo" } },
                { "type": "tool_use", "id": "t2", "name": "Read", "input": { "file_path": "/p/a.rs", "offset": 10, "limit": 31 } },
                { "type": "tool_use", "id": "t3", "name": "Grep", "input": { "pattern": "foo", "path": "src", "output_mode": "content" } },
                { "type": "tool_use", "id": "t4", "name": "WebSearch", "input": { "query": "tokio util" } },
                { "type": "tool_use", "id": "t5", "name": "TodoWrite", "input": { "todos": [{}, {}] } },
                { "type": "tool_use", "id": "t6", "name": "AskUserQuestion", "input": { "questions": [{ "question": "用哪个版本？" }] } },
                { "type": "tool_use", "id": "t7", "name": "Skill", "input": { "skill": "commit" } },
                { "type": "tool_use", "id": "t8", "name": "Agent", "input": { "description": "Review README", "model": "fable", "prompt": "x" } }
            ]),
        )]);
        assert_eq!(
            msgs[0].content,
            [
                "[Tool: Bash] rg -n foo src…",
                "[Tool: Read] /p/a.rs",
                "[Tool: Grep] foo in src",
                "[Tool: WebSearch] tokio util",
                "[Tool: TodoWrite] 2 项",
                "[Tool: AskUserQuestion] 用哪个版本？",
                "[Tool: Skill] Skill",
                "[Tool: Agent] Review README",
            ]
            .join("\n\n")
        );
        let details: Vec<(ToolKind, Option<String>)> = msgs[0]
            .blocks
            .iter()
            .map(|b| match b {
                SessionBlock::ToolCall { kind, detail, .. } => (*kind, detail.clone()),
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(
            details,
            vec![
                (ToolKind::Search, Some("Search foo".to_string())),
                (ToolKind::Read, Some(":10-40".to_string())),
                (ToolKind::Search, Some("content".to_string())),
                (ToolKind::Web, None),
                (ToolKind::Todo, None),
                (ToolKind::Ask, None),
                (ToolKind::Other, Some("commit".to_string())),
                (ToolKind::Agent, Some("fable".to_string())),
            ]
        );
    }

    #[test]
    fn large_tool_results_only_carry_preview_and_refs() {
        let long: String = (1..=200).map(|i| format!("line {i}\n")).collect();
        let mut interrupted = user(
            "u3",
            json!([{ "type": "tool_result", "tool_use_id": "toolu_b", "content": long }]),
        );
        let temp_marker = "__SIDECAR__";
        interrupted["toolUseResult"] = json!({
            "stdout": long, "stderr": "", "interrupted": true,
            "persistedOutputPath": format!("{temp_marker}/tool-results/toolu_b.txt")
        });
        let (temp, _p, msgs, offsets) = {
            // persistedOutputPath 需要真实的附属目录前缀，先算出路径再替换
            let temp = tempdir().expect("tempdir");
            let path = temp.path().join("session.jsonl");
            let sidecar = path.with_extension("");
            // 替换发生在序列化之后，路径要按 JSON 字符串转义（Windows 的反斜杠）
            let sidecar_json = serde_json::to_string(&sidecar.to_string_lossy()).unwrap();
            let sidecar_json = &sidecar_json[1..sidecar_json.len() - 1];
            let lines = [
                user("u1", json!("跑测试")),
                assistant(
                    "a1",
                    "m1",
                    json!([
                        { "type": "tool_use", "id": "toolu_a", "name": "Bash", "input": { "command": "cargo build" } },
                        { "type": "tool_use", "id": "toolu_b", "name": "Bash", "input": { "command": "cargo test", "description": "x".repeat(2000) } }
                    ]),
                ),
                {
                    let mut v = user(
                        "u2",
                        json!([{ "type": "tool_result", "tool_use_id": "toolu_a", "is_error": true, "content": format!("Exit code 101\n{long}") }]),
                    );
                    v["timestamp"] = json!("2026-10-03T10:00:13Z");
                    v
                },
                interrupted,
            ];
            let mut body = String::new();
            let mut offsets = Vec::new();
            for line in &lines {
                offsets.push(body.len() as u64);
                body.push_str(
                    &serde_json::to_string(line)
                        .unwrap()
                        .replace(temp_marker, sidecar_json),
                );
                body.push('\n');
            }
            std::fs::write(&path, body).unwrap();
            let msgs = load_messages(&path).unwrap();
            (temp, path, msgs, offsets)
        };
        let _ = temp;

        let SessionBlock::ToolCall {
            input_full,
            input_total_len,
            input_preview,
            ..
        } = &msgs[1].blocks[1]
        else {
            panic!("expected tool_call");
        };
        assert_eq!(input_preview.chars().count(), INPUT_PREVIEW_CHARS);
        assert!(*input_total_len as usize > INPUT_PREVIEW_CHARS);
        assert!(matches!(
            input_full,
            Some(ContentRef::Jsonl { offset, pointer, .. }) if *offset == offsets[1] && pointer == "/message/content/1/input"
        ));

        let SessionBlock::ToolResult {
            status,
            preview,
            truncated,
            line_count,
            full,
            exit_code,
            duration_ms,
            ..
        } = &msgs[2].blocks[0]
        else {
            panic!("expected tool_result");
        };
        assert_eq!(*status, ToolStatus::Error);
        assert_eq!(*exit_code, Some(101));
        assert_eq!(*duration_ms, Some(12_000));
        assert!(*truncated);
        assert_eq!(*line_count, 201);
        assert_eq!(preview.lines().count(), 12);
        assert!(matches!(
            full,
            Some(ContentRef::Jsonl { offset, pointer, .. }) if *offset == offsets[2] && pointer == "/message/content/0/content"
        ));

        let SessionBlock::ToolResult {
            status,
            full,
            saved_path,
            exit_code,
            ..
        } = &msgs[3].blocks[0]
        else {
            panic!("expected tool_result");
        };
        assert_eq!(*status, ToolStatus::Interrupted);
        assert_eq!(*exit_code, None);
        assert!(saved_path
            .as_deref()
            .unwrap()
            .ends_with("tool-results/toolu_b.txt"));
        assert_eq!(
            full,
            &Some(ContentRef::Sidecar {
                rel_path: "tool-results/toolu_b.txt".to_string()
            })
        );
    }

    /// 与契约 fixture `tests/fixtures/sessions/claude.messages.json` 同构的会话：
    /// 解析结果的「形状」（角色、轮次、注入标记、块类型/kind/status 与字段集合）必须一致。
    #[test]
    fn output_shape_matches_shared_fixture() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/sessions/claude.messages.json"
        ))
        .unwrap();

        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");
        let sidecar = path.with_extension("");
        let img = "iVBORw0KGgo".repeat(400);
        let long_err: String = std::iter::once("Exit code 101".to_string())
            .chain((1..=40).map(|i| format!("error line {i}")))
            .collect::<Vec<_>>()
            .join("\n");
        let long_read: String = (1..=40).map(|i| format!("{i:>6}\tline {i}\n")).collect();
        let long_test: String = (1..=231).map(|i| format!("test {i} ... ok\n")).collect();

        let mut ts = 0;
        let mut at = |mut v: Value| {
            ts += 1;
            v["timestamp"] = json!(format!("2026-10-03T10:{:02}:{:02}Z", ts / 60, ts % 60));
            v
        };
        let usage = json!({ "input_tokens": 3, "output_tokens": 214, "cache_read_input_tokens": 18432, "cache_creation_input_tokens": 2048 });
        let ai = |uuid: &str, id: &str, content: Value| {
            let mut v = assistant(uuid, id, content);
            v["message"]["usage"] = usage.clone();
            v["message"]["stop_reason"] = json!("tool_use");
            v
        };
        let result = |uuid: &str, call: &str, content: Value| {
            user(
                uuid,
                json!([{ "type": "tool_result", "tool_use_id": call, "content": content }]),
            )
        };
        let tool = |id: &str, name: &str, input: Value| json!({ "type": "tool_use", "id": id, "name": name, "input": input });

        let mut lines = vec![
            at(user("u01", json!([
                { "type": "text", "text": "cargo build 报错，帮我看看。" },
                { "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": img } }
            ]))),
            at(user("u02", json!("<system-reminder>\nThe user opened src/net/client.rs\n</system-reminder>"))),
            at({
                let mut v = ai("u03", "msg_a", json!([{ "type": "thinking", "thinking": "", "signature": "sig" }]));
                v["thinkingDurationMs"] = json!(3200);
                v
            }),
            at(ai("u03b", "msg_a", json!([{ "type": "text", "text": "我先跑一遍构建。" }]))),
            at(ai("u03c", "msg_a", json!([tool("toolu_01Bash", "Bash", json!({ "command": "cargo build 2>&1 | tail -50", "description": "Build the project" }))]))),
            at({
                let mut v = user("u04", json!([{ "type": "tool_result", "tool_use_id": "toolu_01Bash", "is_error": true, "content": long_err }]));
                v["toolUseResult"] = json!("Error: Exit code 101");
                v
            }),
            at({
                let mut v = ai("u05", "msg_b", json!([{ "type": "thinking", "thinking": "缺少 tokio-util 依赖。", "signature": "sig" }]));
                v["thinkingDurationMs"] = json!(1800);
                v
            }),
            at(ai("u05b", "msg_b", json!([
                tool("toolu_02Read", "Read", json!({ "file_path": "/p/Cargo.toml", "offset": 1, "limit": 40 })),
                tool("toolu_03Grep", "Grep", json!({ "pattern": "tokio_util", "path": "src", "output_mode": "content" }))
            ]))),
            at(result("u06", "toolu_02Read", json!(long_read))),
            at(result("u07", "toolu_03Grep", json!("src/net/client.rs:7:use tokio_util::codec::LinesCodec;"))),
            at(ai("u08", "msg_c", json!([
                { "type": "text", "text": "补上依赖。" },
                tool("toolu_04Edit", "Edit", json!({ "file_path": "/p/Cargo.toml", "old_string": "a", "new_string": "a\nb" })),
                tool("toolu_05Edit", "Edit", json!({ "file_path": "/p/src/net/client.rs", "old_string": "x", "new_string": "y\nz" }))
            ]))),
            at({
                let mut v = result("u09", "toolu_04Edit", json!("The file /p/Cargo.toml has been updated successfully."));
                v["toolUseResult"] = json!({ "filePath": "/p/Cargo.toml", "structuredPatch": [{ "lines": [" a", "+b"] }] });
                v
            }),
            at(result("u0a", "toolu_05Edit", json!("The file /p/src/net/client.rs has been updated successfully."))),
            at(ai("u0b", "msg_d", json!([tool("toolu_06Bash", "Bash", json!({ "command": "cargo build", "description": "Rebuild" }))]))),
            at({
                let mut v = result("u0c", "toolu_06Bash", json!("Finished `dev` profile"));
                v["toolUseResult"] = json!({ "stdout": "Finished `dev` profile", "stderr": "", "interrupted": false });
                v
            }),
            at(ai("u0d", "msg_e", json!([tool("toolu_07Mcp", "mcp__claude-in-chrome__computer", json!({ "action": "screenshot", "tabId": 1820356711 }))]))),
            at(result("u0e", "toolu_07Mcp", json!([
                { "type": "text", "text": "Successfully captured screenshot" },
                { "type": "image", "source": { "type": "base64", "media_type": "image/jpeg", "data": img } }
            ]))),
            at(ai("u0f", "msg_f", json!([{ "type": "text", "text": "构建已经通过。" }]))),
            at(json!({ "type": "pr-link", "prNumber": 128, "prUrl": "https://github.com/o/r/pull/128" })),
            at(user("u10", json!("<command-name>/compact</command-name>\n<command-message>compact</command-message>\n<command-args></command-args>"))),
            at(json!({ "type": "system", "subtype": "compact_boundary", "content": "Conversation compacted", "uuid": "s1" })),
            at(json!({ "type": "user", "isCompactSummary": true, "uuid": "u10b", "message": { "role": "user", "content": "Conversation compacted: fixed tokio-util." } })),
            at(user("u11", json!("再把全部测试跑一遍。"))),
            at(ai("u12", "msg_g", json!([
                tool("toolu_08Todo", "TodoWrite", json!({ "todos": [{ "content": "a" }, { "content": "b" }] })),
                tool("toolu_09Agent", "Agent", json!({ "description": "Review README install steps", "subagent_type": "Explore", "model": "fable", "prompt": "Check README." })),
                tool("toolu_10Bash", "Bash", json!({ "command": "cargo test --workspace", "description": "Run all tests", "timeout": 600000 }))
            ]))),
            at(result("u13", "toolu_08Todo", json!("Todos have been modified successfully."))),
            at({
                let mut v = result("u14", "toolu_10Bash", json!(long_test));
                v["toolUseResult"] = json!({
                    "interrupted": true,
                    "persistedOutputPath": format!("{}/tool-results/toolu_10Bash.txt", sidecar.display())
                });
                v
            }),
            at(user("u15", json!([{ "type": "text", "text": "[Request interrupted by user for tool use]" }]))),
            at(json!({ "type": "system", "subtype": "stop_hook_summary", "hookErrors": ["Stop hook failed: notify.sh exited with code 127"] })),
            at(user("u16", json!("继续，把 CHANGELOG 也补一条。"))),
            at(ai("u17", "msg_h", json!([tool("toolu_11Ask", "AskUserQuestion", json!({ "questions": [{ "question": "CHANGELOG 用哪个版本号？" }] }))]))),
            at(result("u18", "toolu_11Ask", json!("User has answered your questions."))),
            at(ai("u19", "msg_i", json!([
                tool("toolu_12Web", "WebFetch", json!({ "url": "https://keepachangelog.com/en/1.1.0/", "prompt": "Summarize" })),
                tool("toolu_13Write", "Write", json!({ "file_path": "/p/CHANGELOG.md", "content": "# Changelog\n\n## [0.3.2]\n" }))
            ]))),
            at(result("u1a", "toolu_12Web", json!("Added, Changed, Fixed."))),
            at({
                let mut v = result("u1b", "toolu_13Write", json!("File created successfully at: /p/CHANGELOG.md"));
                v["toolUseResult"] = json!({ "type": "create", "filePath": "/p/CHANGELOG.md", "structuredPatch": [] });
                v
            }),
            at(ai("u1c", "msg_j", json!([{ "type": "text", "text": "已新增 0.3.2 条目。" }]))),
        ];
        // 噪声记录：应被跳过
        lines.insert(
            1,
            json!({ "type": "attachment", "attachment": { "type": "deferred_tools_delta" } }),
        );
        lines.push(json!({ "type": "custom-title", "customTitle": "x" }));
        lines.push(json!({ "type": "last-prompt", "lastPrompt": "x" }));

        let body: String = lines
            .iter()
            .map(|l| serde_json::to_string(l).unwrap() + "\n")
            .collect();
        std::fs::write(&path, body).unwrap();
        let parsed = serde_json::to_value(load_messages(&path).unwrap()).unwrap();

        /// 形状：角色、轮次、注入标记、有无 id/meta，以及每个块的类型、判别值和字段集合
        fn shape(messages: &Value) -> Vec<Value> {
            messages
                .as_array()
                .unwrap()
                .iter()
                .map(|m| {
                    let blocks: Vec<Value> = m["blocks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|b| {
                            let mut keys: Vec<&str> =
                                b.as_object().unwrap().keys().map(String::as_str).collect();
                            keys.sort_unstable();
                            json!({
                                "type": b["type"], "kind": b["kind"], "status": b["status"],
                                "rawName": b["rawName"], "redacted": b["redacted"], "keys": keys,
                                "full": b["full"]["kind"], "imageKinds": b["images"].as_array().map(|i| i.len()),
                                "diffOp": b["diff"]["files"][0]["op"],
                            })
                        })
                        .collect();
                    json!({
                        "role": m["role"], "turnId": m["turnId"], "injected": m.get("injected"),
                        "hasId": m.get("id").is_some(), "hasMeta": m.get("meta").is_some(),
                        "blocks": blocks,
                    })
                })
                .collect()
        }

        let expected = shape(&fixture);
        let actual = shape(&parsed);
        assert_eq!(actual.len(), expected.len(), "消息条数");
        for (i, (a, e)) in actual.iter().zip(&expected).enumerate() {
            assert_eq!(a, e, "第 {i} 条消息形状不一致");
        }

        // 回填与配对
        let messages: Vec<SessionMessage> = serde_json::from_value(parsed).unwrap();
        let call_ids: HashSet<&str> = messages
            .iter()
            .flat_map(|m| &m.blocks)
            .filter_map(|b| match b {
                SessionBlock::ToolCall { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        for block in messages.iter().flat_map(|m| &m.blocks) {
            if let SessionBlock::ToolResult { call_id, .. } = block {
                assert!(call_ids.contains(call_id.as_str()), "{call_id} 未配对");
            }
        }
        // 有 blocks 的消息不下发 content（前端从 blocks 推导）
        assert!(messages.iter().all(|m| m.content.is_empty()));
    }
}
