import { FormEvent, useEffect, useState, useRef } from "react";
import { cn } from "../lib/utils";
import { AnimatedButton } from "./AnimatedButton";
import { Avatar } from "./Avatar";
import { MentionPicker } from "./MentionPicker";
import { findMentionAtCursor, insertMention, type MentionMatch } from "../lib/mentions";
import { listActors, type Actor } from "../lib/client";

interface ComposerProps {
  onSubmit: (body: string, files: File[]) => Promise<void>;
  onOpenProfile?: () => void;
  placeholder?: string;
  variant?: "default" | "minimal" | "reply";
  replyTo?: string | null;
  avatar?: string;
  userHandle?: string;
  focusKey?: number;
}

const MAX_CHARS = 280;
const MAX_FILES = 4;
const ACCEPTED_FILES = ".pdf,.docx,.pptx,.xlsx,.csv,.png,.jpg,.jpeg,.webp";

export function Composer({
  onSubmit,
  onOpenProfile,
  placeholder = "What is happening?!",
  variant = "default",
  replyTo,
  avatar,
  userHandle = "You",
  focusKey,
}: ComposerProps) {
  const [body, setBody] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [agents, setAgents] = useState<Actor[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [mentionMatch, setMentionMatch] = useState<MentionMatch | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const charCount = body.length;
  const charPercentage = (charCount / MAX_CHARS) * 100;
  const isOverLimit = charCount > MAX_CHARS;

  useEffect(() => {
    if (focusKey == null) return;
    textareaRef.current?.focus();
  }, [focusKey]);

  useEffect(() => {
    void listActors()
      .then((actors) => setAgents(actors.filter((actor) => actor.kind === "agent")))
      .catch(() => setAgents([]));
  }, []);

  const resizeTextarea = () => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  };

  const updateMentionMatch = (value = body) => {
    const cursor = textareaRef.current?.selectionStart ?? value.length;
    setMentionMatch(findMentionAtCursor(value, cursor));
  };

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = body.trim();
    if (!trimmed || isOverLimit) return;

    setSubmitting(true);
    try {
      await onSubmit(trimmed, selectedFiles);
      setBody("");
      setSelectedFiles([]);
      if (fileInputRef.current) fileInputRef.current.value = "";
      setMentionMatch(null);
    } finally {
      setSubmitting(false);
    }
  }

  // 自动调整高度
  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const nextValue = e.target.value;
    setBody(nextValue);
    setMentionMatch(findMentionAtCursor(nextValue, e.target.selectionStart));
    resizeTextarea();
  };

  const handleSelectMention = (handle: string) => {
    if (!mentionMatch) return;
    const { nextValue, cursor } = insertMention(body, mentionMatch, handle);
    setBody(nextValue);
    setMentionMatch(null);
    requestAnimationFrame(() => {
      textareaRef.current?.focus();
      textareaRef.current?.setSelectionRange(cursor, cursor);
      resizeTextarea();
    });
  };

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const incoming = Array.from(event.target.files ?? []);
    if (incoming.length === 0) return;
    setSelectedFiles((current) => {
      const next = [...current, ...incoming].slice(0, MAX_FILES);
      if (current.length + incoming.length > MAX_FILES) {
        window.alert(`最多只能附加 ${MAX_FILES} 个文件。`);
      }
      return next;
    });
    event.target.value = "";
  };

  const removeSelectedFile = (index: number) => {
    setSelectedFiles((current) => current.filter((_, currentIndex) => currentIndex !== index));
  };

  const renderAvatar = (size: "sm" | "md" = "md") => {
    const sizeClass = size === "sm" ? "h-10 w-10" : "h-12 w-12";
    
    if (avatar?.startsWith("data:")) {
      return (
        <img
          src={avatar}
          alt={userHandle}
          className={cn(
            sizeClass,
            "aspect-square shrink-0 rounded-full border-2 border-x-background object-cover dark:border-x-background-dark"
          )}
        />
      );
    }
    
    return (
      <Avatar seed={avatar || userHandle} label={userHandle} size={size === "sm" ? "sm" : "md"} />
    );
  };

  if (variant === "minimal") {
    return (
      <div
        onClick={() => textareaRef.current?.focus()}
        className={cn(
          "flex cursor-text items-center gap-3 border-b border-x-border dark:border-x-border-dark px-4 py-3",
          "transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
        )}
      >
        <div
          onClick={(e) => {
            e.stopPropagation();
            onOpenProfile?.();
          }}
          className="h-10 w-10 flex-none self-start overflow-hidden rounded-full cursor-pointer transition-all hover:ring-2 hover:ring-x-primary/30"
        >
          {renderAvatar("sm")}
        </div>
        <span className="text-x-text-secondary">{placeholder}</span>
      </div>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      className={cn(
        "border-b border-x-border dark:border-x-border-dark",
        variant === "reply" ? "" : "px-4 py-3"
      )}
    >
      {replyTo && (
        <div className="px-4 pt-3 text-sm text-x-text-secondary">
          Replying to <span className="text-x-primary">@{replyTo}</span>
        </div>
      )}
      
      <div className="flex gap-3 px-4 py-3">
        {/* 头像 */}
        <div
          onClick={onOpenProfile}
          className="h-12 w-12 flex-none self-start overflow-hidden rounded-full cursor-pointer transition-all hover:ring-2 hover:ring-x-primary/30"
        >
          {renderAvatar("md")}
        </div>

        {/* 输入区 */}
        <div className="min-w-0 flex-1">
          <div className="relative">
            <textarea
              ref={textareaRef}
              value={body}
              onChange={handleInput}
              onClick={() => updateMentionMatch()}
              onKeyUp={(event) => {
                if (event.key !== "Escape") updateMentionMatch();
              }}
              onKeyDown={(event) => {
                if (event.key === "Escape") setMentionMatch(null);
              }}
              placeholder={placeholder}
              rows={1}
              className={cn(
                "w-full resize-none border-none bg-transparent text-xl leading-normal outline-none placeholder:text-x-text-secondary",
                "text-x-text dark:text-x-text-dark"
              )}
              style={{ minHeight: "24px" }}
            />
            {mentionMatch && (
              <MentionPicker
                agents={agents}
                query={mentionMatch.query}
                onSelect={handleSelectMention}
              />
            )}
          </div>

          {/* 分割线 */}
          <div className="my-3 border-t border-x-border dark:border-x-border-dark" />

          {selectedFiles.length > 0 && (
            <div className="mb-3 flex flex-wrap gap-2">
              {selectedFiles.map((file, index) => (
                <button
                  key={`${file.name}-${file.size}-${index}`}
                  type="button"
                  onClick={() => removeSelectedFile(index)}
                  className="max-w-full rounded-full border border-x-border bg-x-surface-hover px-3 py-1 text-xs font-semibold text-x-text-secondary transition-colors hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-600 dark:border-x-border-dark dark:bg-x-surface-hover-dark"
                  title="Remove file"
                >
                  <span className="inline-block max-w-[180px] truncate align-bottom">{file.name}</span>
                  <span className="ml-2">x</span>
                </button>
              ))}
            </div>
          )}

          {/* 底部工具栏 */}
          <div className="flex items-center justify-between gap-3">
            <div>
              <input
                ref={fileInputRef}
                type="file"
                multiple
                accept={ACCEPTED_FILES}
                className="hidden"
                onChange={handleFileSelect}
              />
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="group flex h-9 w-9 items-center justify-center rounded-full text-x-text-secondary transition-colors hover:bg-x-primary/10 hover:text-x-primary"
                aria-label="Attach files"
              >
                <svg viewBox="0 0 24 24" className="h-5 w-5" fill="none" stroke="currentColor" strokeWidth={1.8}>
                  <path d="M21.44 11.05 12.2 20.29a6 6 0 0 1-8.49-8.49l9.9-9.9a4 4 0 1 1 5.66 5.66l-9.9 9.9a2 2 0 0 1-2.83-2.83l8.49-8.49" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </button>
            </div>
            {/* 发送按钮区 */}
            <div className="flex items-center gap-3">
              {/* 字符计数 */}
              {charCount > 0 && (
                <div className="relative flex items-center">
                  <svg className="h-6 w-6 -rotate-90" viewBox="0 0 36 36">
                    <circle
                      cx="18"
                      cy="18"
                      r="16"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      className="text-x-border dark:text-x-border-dark"
                    />
                    <circle
                      cx="18"
                      cy="18"
                      r="16"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeDasharray={`${charPercentage * 1.01} 100`}
                      className={cn(
                        "transition-all duration-300",
                        isOverLimit ? "text-red-500" : charPercentage > 80 ? "text-yellow-500" : "text-x-primary"
                      )}
                    />
                  </svg>
                  {charPercentage > 100 && (
                    <span className="absolute inset-0 flex items-center justify-center text-xs font-medium text-red-500">
                      {MAX_CHARS - charCount}
                    </span>
                  )}
                </div>
              )}

              <AnimatedButton
                type="submit"
                disabled={submitting || !body.trim() || isOverLimit}
                size="sm"
              >
                {submitting ? "Posting..." : "Post"}
              </AnimatedButton>
            </div>
          </div>
        </div>
      </div>
    </form>
  );
}
