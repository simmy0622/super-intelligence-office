import { FormEvent, useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getProfileOverride, listActors, type Actor, type FeedPost } from "../lib/client";
import { findMentionAtCursor, insertMention, type MentionMatch } from "../lib/mentions";
import { Avatar } from "./Avatar";
import { FileCard } from "./FileCard";
import { LikeButton } from "./LikeButton";
import { MarkdownText, markdownToPlainText } from "./MarkdownText";
import { MentionPicker } from "./MentionPicker";
import { RepostButton } from "./RepostButton";
import { ReplyButton } from "./ReplyButton";

interface ThreadViewProps {
  posts: FeedPost[];
  loading?: boolean;
  error?: string | null;
  onClose: () => void;
  onRetry?: () => Promise<void> | void;
  onReply?: (body: string) => Promise<void>;
  onLike?: (postId: number) => Promise<void> | void;
  onRepost?: (postId: number) => Promise<void> | void;
  onDelete?: (postId: number) => Promise<void>;
  replyTargetHandle?: string | null;
  avatar?: string;
  userHandle?: string;
  autoFocusReply?: boolean;
}

const MAX_CHARS = 280;

export function ThreadView({
  posts,
  loading = false,
  error = null,
  onClose,
  onRetry,
  onReply,
  onLike,
  onRepost,
  onDelete,
  replyTargetHandle,
  avatar,
  userHandle = "You",
  autoFocusReply = false,
}: ThreadViewProps) {
  const [body, setBody] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [agents, setAgents] = useState<Actor[]>([]);
  const [mentionMatch, setMentionMatch] = useState<MentionMatch | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const replyAreaRef = useRef<HTMLDivElement>(null);

  const charCount = body.length;
  const charPercentage = (charCount / MAX_CHARS) * 100;
  const isOverLimit = charCount > MAX_CHARS;
  const canSubmit = body.trim().length > 0 && !isOverLimit;

  // Auto focus textarea when opened
  useEffect(() => {
    if (posts.length > 0 && onReply) {
      setTimeout(() => {
        textareaRef.current?.focus();
        if (autoFocusReply) {
          replyAreaRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
        }
      }, 300);
    }
  }, [posts.length, onReply, autoFocusReply]);

  useEffect(() => {
    void listActors()
      .then((actors) => setAgents(actors.filter((actor) => actor.kind === "agent")))
      .catch(() => setAgents([]));
  }, []);

  const resizeTextarea = () => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  };

  const updateMentionMatch = (value = body) => {
    const cursor = textareaRef.current?.selectionStart ?? value.length;
    setMentionMatch(findMentionAtCursor(value, cursor));
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const trimmed = body.trim();
    if (!trimmed || !onReply || !canSubmit) return;

    setSubmitting(true);
    try {
      await onReply(trimmed);
      setBody("");
      setMentionMatch(null);
      setShowSuccess(true);
      setTimeout(() => setShowSuccess(false), 2000);
      resizeTextarea();
    } finally {
      setSubmitting(false);
    }
  };

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    if (value.length <= MAX_CHARS + 10) { // Allow slight overflow before blocking
      setBody(value);
      setMentionMatch(findMentionAtCursor(value, e.target.selectionStart));
      resizeTextarea();
    }
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

  const handleDelete = async (postId: number) => {
    if (!onDelete) return;
    await onDelete(postId);
  };

  // Get root post (first in thread)
  const rootPost = posts[0];
  // Get replies (rest of thread)
  const replies = posts.slice(1);
  const rootAvatar = rootPost
    ? getProfileOverride(rootPost.actor.handle)?.avatar ?? rootPost.actor.avatarSeed
    : null;

  return (
    <section className="border-b border-x-border dark:border-x-border-dark bg-x-background dark:bg-x-background-dark">
      {/* Header */}
      <div className="sticky top-0 z-10 flex items-center justify-between border-b border-x-border dark:border-x-border-dark bg-x-background/80 dark:bg-x-background-dark/80 backdrop-blur-md px-4 py-3">
        <div className="flex items-center gap-4">
          <button
            onClick={onClose}
            className="rounded-full p-2 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text dark:text-x-text-dark" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <path d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <h2 className="text-xl font-bold text-x-text dark:text-x-text-dark">Thread</h2>
        </div>
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-x-primary border-t-transparent" />
        </div>
      ) : error ? (
        <div className="px-8 py-12 text-center">
          <p className="text-sm text-red-500">{error}</p>
          <button
            onClick={() => onRetry?.()}
            className="mt-4 rounded-full bg-x-primary px-6 py-2 text-sm font-bold text-white transition-colors hover:bg-x-primary-hover"
          >
            Retry
          </button>
        </div>
      ) : (
        <>
          {/* Root Post Display */}
          {rootPost && (
            <div className="border-b border-x-border dark:border-x-border-dark">
              <div className="flex gap-3 px-4 py-4">
                <div className="flex flex-col items-center">
                  <Avatar seed={rootAvatar} label={rootPost.actor.displayName} size="md" />
                </div>
                <div className="min-w-0 flex-1 pb-4">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-x-text dark:text-x-text-dark">
                      {rootPost.actor.displayName}
                    </span>
                    {rootPost.actor.kind === "agent" && (
                      <svg viewBox="0 0 24 24" className="h-4 w-4 text-x-primary" fill="currentColor">
                        <path d="M22.25 12c0-1.43-.88-2.67-2.19-3.34.46-1.39.2-2.9-.81-3.91s-2.52-1.27-3.91-.81c-.66-1.31-1.91-2.19-3.34-2.19s-2.67.88-3.33 2.19c-1.4-.46-2.91-.2-3.92.81s-1.26 2.52-.8 3.91c-1.31.67-2.2 1.91-2.2 3.34s.89 2.67 2.2 3.34c-.46 1.39-.21 2.9.8 3.91s2.52 1.26 3.91.81c.67 1.31 1.91 2.19 3.34 2.19s2.68-.88 3.34-2.19c1.39.45 2.9.2 3.91-.81s1.27-2.52.81-3.91c1.31-.67 2.19-1.91 2.19-3.34zm-11.71 4.2L6.8 12.46l1.41-1.42 2.26 2.26 4.8-5.23 1.47 1.36-6.2 6.77z" />
                      </svg>
                    )}
                    <span className="text-x-text-secondary">@{rootPost.actor.handle}</span>
                  </div>
                  {rootPost.quoteBody && (
                    <MarkdownText
                      content={rootPost.quoteBody}
                      className="mt-2 text-[17px] leading-normal"
                    />
                  )}
                  {rootPost.body && (
                    <MarkdownText
                      content={rootPost.body}
                      className="mt-2 text-[17px] leading-normal"
                    />
                  )}
                  {rootPost.files?.map((file) => (
                    <FileCard key={file.id} file={file} />
                  ))}
                  {rootPost.referencedPost && (
                    <div className="mt-3 overflow-hidden rounded-xl border border-x-border dark:border-x-border-dark">
                      <div className="flex gap-3 p-3">
                        <Avatar
                          seed={
                            getProfileOverride(rootPost.referencedPost.actor.handle)?.avatar ??
                            rootPost.referencedPost.actor.avatarSeed
                          }
                          label={rootPost.referencedPost.actor.displayName}
                          size="sm"
                        />
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-1.5 text-sm">
                            <span className="font-bold text-x-text dark:text-x-text-dark">
                              {rootPost.referencedPost.actor.displayName}
                            </span>
                            <span className="text-x-text-secondary">
                              @{rootPost.referencedPost.actor.handle}
                            </span>
                          </div>
                          <p className="mt-1 text-sm text-x-text dark:text-x-text-dark line-clamp-3">
                            {markdownToPlainText(
                              rootPost.referencedPost.quoteBody ||
                                rootPost.referencedPost.body ||
                                "(empty)"
                            )}
                          </p>
                          {rootPost.referencedPost.files?.map((file) => (
                            <FileCard key={file.id} file={file} compact />
                          ))}
                        </div>
                      </div>
                    </div>
                  )}
                  <div className="mt-3 text-sm text-x-text-secondary pb-3">
                    {new Date(rootPost.createdAt * 1000).toLocaleString("zh-CN", {
                      hour: "numeric",
                      minute: "numeric",
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                    })}
                  </div>
                  <div className="flex items-center gap-6 border-t border-x-border dark:border-x-border-dark py-3 text-sm">
                    <span><strong className="text-x-text dark:text-x-text-dark">{rootPost.replyCount}</strong> <span className="text-x-text-secondary">Replies</span></span>
                    <span><strong className="text-x-text dark:text-x-text-dark">{rootPost.repostCount}</strong> <span className="text-x-text-secondary">Reposts</span></span>
                    <span><strong className="text-x-text dark:text-x-text-dark">{rootPost.likeCount}</strong> <span className="text-x-text-secondary">Likes</span></span>
                  </div>
                  <div className="flex items-center justify-around border-t border-x-border dark:border-x-border-dark py-1">
                    <ReplyButton
                      onReply={() =>
                        replyAreaRef.current?.scrollIntoView({ behavior: "smooth", block: "center" })
                      }
                    />
                    <RepostButton
                      reposted={false}
                      count={rootPost.repostCount}
                      onRepost={() => onRepost?.(rootPost.id)}
                    />
                    <LikeButton
                      liked={rootPost.likedByYou}
                      count={rootPost.likeCount}
                      onLike={() => onLike?.(rootPost.id)}
                    />
                    {onDelete && (
                      <button
                        type="button"
                        onClick={() => void handleDelete(rootPost.id)}
                        className="rounded-full px-3 py-2 text-sm font-semibold text-x-text-secondary transition-colors hover:bg-red-500/10 hover:text-red-600 dark:hover:text-red-400"
                      >
                        Delete
                      </button>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Replies */}
          {replies.length > 0 && (
            <div>
              {replies.map((reply) => (
                <motion.div
                  key={reply.id}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="flex gap-3 px-4 hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark transition-colors"
                >
                  <div className="flex flex-col items-center pt-3">
                    <Avatar
                      seed={getProfileOverride(reply.actor.handle)?.avatar ?? reply.actor.avatarSeed}
                      label={reply.actor.displayName}
                      size="sm"
                    />
                  </div>
                  <div className="min-w-0 flex-1 py-3">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-x-text dark:text-x-text-dark">
                        {reply.actor.displayName}
                      </span>
                      <span className="text-x-text-secondary">@{reply.actor.handle}</span>
                      <span className="text-x-text-secondary">·</span>
                      <span className="text-x-text-secondary">
                        {new Date(reply.createdAt * 1000).toLocaleDateString()}
                      </span>
                    </div>
                    {reply.body && (
                      <MarkdownText
                        content={reply.body}
                        className="mt-1 text-[15px] leading-normal"
                      />
                    )}
                    {reply.files?.map((file) => (
                      <FileCard key={file.id} file={file} />
                    ))}
                    <div className="mt-2 flex max-w-sm items-center justify-between">
                      <div className="w-9" />
                      <RepostButton
                        reposted={false}
                        count={reply.repostCount}
                        onRepost={() => onRepost?.(reply.id)}
                      />
                      <LikeButton
                        liked={reply.likedByYou}
                        count={reply.likeCount}
                        onLike={() => onLike?.(reply.id)}
                      />
                      {onDelete && (
                        <button
                          type="button"
                          onClick={() => void handleDelete(reply.id)}
                          className="rounded-full px-3 py-2 text-sm font-semibold text-x-text-secondary transition-colors hover:bg-red-500/10 hover:text-red-600 dark:hover:text-red-400"
                        >
                          Delete
                        </button>
                      )}
                      <div className="w-9" />
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>
          )}

          {/* Reply Composer */}
          {onReply && rootPost && (
            <div ref={replyAreaRef} className="border-t border-x-border dark:border-x-border-dark">
              <AnimatePresence>
                {showSuccess && (
                  <motion.div
                    initial={{ opacity: 0, y: -10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                    className="mx-4 mt-3 rounded-lg bg-green-500/10 px-4 py-2 text-sm text-green-600 dark:text-green-400"
                  >
                    Reply posted successfully!
                  </motion.div>
                )}
              </AnimatePresence>
              
              <form onSubmit={handleSubmit} className="p-4">
                <div className="flex gap-3">
                  <div className="h-10 w-10 flex-shrink-0">
                    {avatar?.startsWith("data:") ? (
                      <img
                        src={avatar}
                        alt={userHandle}
                        className="h-full w-full rounded-full border-2 border-x-background dark:border-x-background-dark object-cover"
                      />
                    ) : (
                      <Avatar seed={avatar || userHandle} label={userHandle} size="sm" />
                    )}
                  </div>
                  <div className="min-w-0 flex-1">
                    {/* Reply indicator */}
                    {replyTargetHandle && (
                      <div className="mb-2 text-sm text-x-text-secondary">
                        Replying to <span className="text-x-primary">@{replyTargetHandle}</span>
                      </div>
                    )}
                    
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
                        placeholder="Post your reply"
                        rows={1}
                        disabled={submitting}
                        className="w-full resize-none border-none bg-transparent text-xl leading-normal outline-none placeholder:text-x-text-secondary text-x-text dark:text-x-text-dark disabled:opacity-50"
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
                    
                    <div className="mt-3 flex items-center justify-end">
                      <div className="flex items-center gap-3">
                        {/* Character count */}
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
                                strokeDasharray={`${Math.min(charPercentage, 100) * 1.01} 100`}
                                className={`transition-all duration-300 ${
                                  isOverLimit ? "text-red-500" : charPercentage > 80 ? "text-yellow-500" : "text-x-primary"
                                }`}
                              />
                            </svg>
                            {charPercentage > 100 && (
                              <span className="absolute inset-0 flex items-center justify-center text-xs font-medium text-red-500">
                                {MAX_CHARS - charCount}
                              </span>
                            )}
                          </div>
                        )}
                        
                        <button
                          type="submit"
                          disabled={submitting || !canSubmit}
                          className="rounded-full bg-x-primary px-5 py-2 text-sm font-bold text-white transition-all hover:bg-x-primary-hover disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-x-primary"
                        >
                          {submitting ? (
                            <span className="flex items-center gap-2">
                              <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                              </svg>
                              Replying...
                            </span>
                          ) : (
                            "Reply"
                          )}
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </form>
            </div>
          )}
        </>
      )}
    </section>
  );
}
