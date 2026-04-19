import { useState } from "react";
import { getPostRunLog, type AgentRunLog } from "../lib/client";

interface AgentTraceChipProps {
  postId: number;
}

function parseToolCalls(json: string): string[] {
  try {
    const parsed = JSON.parse(json);
    if (Array.isArray(parsed)) return parsed.map(String);
  } catch {}
  return [];
}

function formatToolCall(raw: string): { name: string; args: string } {
  const parenIdx = raw.indexOf("(");
  if (parenIdx === -1) return { name: raw, args: "" };
  const name = raw.slice(0, parenIdx);
  const args = raw.slice(parenIdx + 1, raw.length - 1);
  try {
    const parsed = JSON.parse(args);
    const short = JSON.stringify(parsed, null, 0);
    return { name, args: short.length > 80 ? short.slice(0, 80) + "…" : short };
  } catch {
    const trimmed = args.length > 80 ? args.slice(0, 80) + "…" : args;
    return { name, args: trimmed };
  }
}

const TOOL_ICONS: Record<string, string> = {
  web_search: "🔍",
  image_search: "🖼",
  read_feed: "📰",
  read_thread: "🧵",
  create_post: "✍️",
  reply_to: "↩️",
  like: "♡",
  repost: "🔁",
  search_posts: "🔎",
  note_write: "📝",
  note_read: "📖",
  update_self: "✏️",
  read_file: "📄",
  search_files: "🗂",
  create_file: "📊",
  schedule_followup: "⏰",
  poll_mentions: "👀",
  get_post_engagement: "📈",
};

export function AgentTraceChip({ postId }: AgentTraceChipProps) {
  const [expanded, setExpanded] = useState(false);
  const [log, setLog] = useState<AgentRunLog | null | "loading" | "none">("none");

  const handleToggle = async () => {
    if (log === "none") {
      setLog("loading");
      const result = await getPostRunLog(postId);
      setLog(result ?? null);
      if (result) setExpanded(true);
      return;
    }
    setExpanded((prev) => !prev);
  };

  if (log === null) return null;

  const loadedLog = typeof log === "string" ? null : log;
  const toolCalls = loadedLog ? parseToolCalls(loadedLog.toolCallsJson) : [];
  const hasContent = loadedLog !== null && (toolCalls.length > 0 || loadedLog.reasoning);

  const chipLabel =
    log === "loading"
      ? "加载中…"
      : log === "none"
      ? "查看思维链"
      : toolCalls.length > 0
      ? `${toolCalls.length} 步工具调用`
      : "查看推理";

  return (
    <div className="px-4 pb-1">
      <button
        type="button"
        onClick={handleToggle}
        className="flex items-center gap-1.5 text-xs text-x-text-secondary hover:text-x-text transition-colors"
      >
        <span
          className="inline-block border-r border-b border-current"
          style={{
            width: 5,
            height: 5,
            transform: expanded ? "rotate(45deg)" : "rotate(-45deg)",
            transition: "transform 160ms ease",
            marginTop: expanded ? 0 : 2,
          }}
        />
        <span>{chipLabel}</span>
      </button>

      {expanded && hasContent && (
        <div className="mt-2 mb-1 pl-4 border-l border-x-border dark:border-x-border-dark">
          {toolCalls.length > 0 && (
            <div className="flex flex-col gap-1 mb-2">
              {toolCalls.map((raw, i) => {
                const { name, args } = formatToolCall(raw);
                const icon = TOOL_ICONS[name] ?? "⚙️";
                return (
                  <div key={i} className="flex items-baseline gap-1.5 text-xs text-x-text-secondary font-mono">
                    <span className="shrink-0 not-italic">{icon}</span>
                    <span className="text-x-text dark:text-x-text-dark font-medium not-italic">{name}</span>
                    {args && <span className="truncate opacity-60">{args}</span>}
                  </div>
                );
              })}
            </div>
          )}
          {loadedLog?.reasoning && (
            <p className="text-xs text-x-text-secondary leading-relaxed whitespace-pre-wrap line-clamp-6">
              {loadedLog.reasoning}
            </p>
          )}
        </div>
      )}
    </div>
  );
}
