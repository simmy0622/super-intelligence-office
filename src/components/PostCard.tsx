import { memo } from "react";
import { motion } from "framer-motion";
import { Link, useNavigate } from "react-router-dom";
import { Avatar } from "./Avatar";
import { FileCard } from "./FileCard";
import { LikeButton } from "./LikeButton";
import { MarkdownText, markdownToPlainText } from "./MarkdownText";
import { RepostButton } from "./RepostButton";
import { ReplyButton } from "./ReplyButton";
import { cn, formatRelativeTime } from "../lib/utils";
import { getProfileOverride, type FeedPost, type PostMedia, type PostReference } from "../lib/client";

interface PostCardProps {
  post: FeedPost;
  onLike?: (postId: number) => Promise<void> | void;
  onRepost?: (postId: number) => Promise<void> | void;
  onDelete?: (postId: number) => Promise<void>;
  onPin?: (postId: number) => void;
  animated?: boolean;
}

function PostMediaGrid({ media, compact = false }: { media: PostMedia[]; compact?: boolean }) {
  const images = media.filter((item) => item.kind === "image" && item.url);
  if (images.length === 0) return null;

  if (compact) {
    const image = images[0];
    return (
      <div className="mt-2 overflow-hidden rounded-xl border border-x-border dark:border-x-border-dark">
        <img
          src={image.thumbnailUrl || image.url}
          alt={image.altText || ""}
          loading="lazy"
          referrerPolicy="no-referrer"
          className="h-28 w-full object-cover"
        />
      </div>
    );
  }

  return (
    <div
      className={cn(
        "pointer-events-auto mt-3 overflow-hidden rounded-2xl border border-x-border bg-x-surface dark:border-x-border-dark dark:bg-x-surface-dark",
        images.length > 1 && "grid grid-cols-2 gap-0.5"
      )}
      onClick={(e) => e.stopPropagation()}
    >
      {images.map((image) => (
        <img
          key={image.id}
          src={image.thumbnailUrl || image.url}
          alt={image.altText || ""}
          loading="lazy"
          referrerPolicy="no-referrer"
          className={cn(
            "w-full object-cover",
            images.length === 1 ? "max-h-[420px]" : "aspect-square"
          )}
        />
      ))}
    </div>
  );
}

function ReferencedPostPreview({ referencedPost }: { referencedPost: PostReference }) {
  const referencedAvatar =
    getProfileOverride(referencedPost.actor.handle)?.avatar ?? referencedPost.actor.avatarSeed;
  return (
    <Link
      to={`/post/${referencedPost.id}`}
      className="pointer-events-auto mt-3 block overflow-hidden rounded-xl border border-x-border transition-colors hover:bg-x-surface-hover dark:border-x-border-dark dark:hover:bg-x-surface-hover-dark"
    >
      <div className="flex gap-3 p-3">
        <Avatar seed={referencedAvatar} label={referencedPost.actor.displayName} size="sm" />
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5 text-sm">
            <span className="font-bold text-x-text dark:text-x-text-dark">
              {referencedPost.actor.displayName}
            </span>
            <span className="text-x-text-secondary">
              @{referencedPost.actor.handle} · {formatRelativeTime(referencedPost.createdAt)}
            </span>
          </div>
          {(referencedPost.quoteBody || referencedPost.body) ? (
            <p className="mt-1 text-sm text-x-text dark:text-x-text-dark line-clamp-3">
              {markdownToPlainText(referencedPost.quoteBody || referencedPost.body || "")}
            </p>
          ) : (
            <p className="mt-1 text-sm text-x-text-secondary italic">Original post unavailable</p>
          )}
          <PostMediaGrid media={referencedPost.media ?? []} compact />
          {referencedPost.files?.map((file) => (
            <FileCard key={file.id} file={file} compact />
          ))}
        </div>
      </div>
    </Link>
  );
}

function PostCardComponent({ post, onLike, onRepost, onDelete, onPin, animated = true }: PostCardProps) {
  const navigate = useNavigate();
  const postAvatar = getProfileOverride(post.actor.handle)?.avatar ?? post.actor.avatarSeed;
  const handleLike = () => onLike?.(post.id);
  const handleRepost = () => onRepost?.(post.id);
  const handleDelete = async () => {
    if (!onDelete) return;
    await onDelete(post.id);
  };
  const handleOpenThread = () => navigate(`/post/${post.id}`);
  const handleOpenReply = () => navigate(`/post/${post.id}?reply=1`);

  return (
    <motion.article
      initial={animated ? { opacity: 0, y: 10 } : false}
      animate={animated ? { opacity: 1, y: 0 } : undefined}
      transition={animated ? { duration: 0.3, ease: "easeOut" } : undefined}
      className={cn(
        "relative border-b border-x-border dark:border-x-border-dark",
        "transition-colors duration-200",
        "hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark",
        post.pinnedAt != null && "bg-x-primary/[0.04] dark:bg-x-primary/[0.08]"
      )}
    >
      <div
        role="link"
        tabIndex={0}
        aria-label={`Open post by ${post.actor.displayName}`}
        onClick={handleOpenThread}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            handleOpenThread();
          }
        }}
        className="absolute inset-0 z-0 cursor-pointer"
      />
      <div className="relative z-10 flex gap-3 px-4 py-3">
        {/* 头像 */}
        <Link
          to={`/profile/${post.actor.handle}`}
          className="pointer-events-auto flex-shrink-0"
          onClick={(e) => e.stopPropagation()}
        >
          <Avatar seed={postAvatar} label={post.actor.displayName} size="md" />
        </Link>

        {/* 内容区 */}
        <div className="pointer-events-none min-w-0 flex-1">
          {/* 头部信息 */}
          <div className="flex items-center gap-1.5">
            <Link
              to={`/profile/${post.actor.handle}`}
              className="pointer-events-auto truncate font-bold text-x-text hover:underline dark:text-x-text-dark"
              onClick={(e) => e.stopPropagation()}
            >
              {post.actor.displayName}
            </Link>
            {post.actor.kind === "agent" && (
              <svg
                viewBox="0 0 24 24"
                className="h-4 w-4 flex-shrink-0 text-x-primary"
                fill="currentColor"
              >
                <path d="M22.25 12c0-1.43-.88-2.67-2.19-3.34.46-1.39.2-2.9-.81-3.91s-2.52-1.27-3.91-.81c-.66-1.31-1.91-2.19-3.34-2.19s-2.67.88-3.33 2.19c-1.4-.46-2.91-.2-3.92.81s-1.26 2.52-.8 3.91c-1.31.67-2.2 1.91-2.2 3.34s.89 2.67 2.2 3.34c-.46 1.39-.21 2.9.8 3.91s2.52 1.26 3.91.81c.67 1.31 1.91 2.19 3.34 2.19s2.68-.88 3.34-2.19c1.39.45 2.9.2 3.91-.81s1.27-2.52.81-3.91c1.31-.67 2.19-1.91 2.19-3.34zm-11.71 4.2L6.8 12.46l1.41-1.42 2.26 2.26 4.8-5.23 1.47 1.36-6.2 6.77z" />
              </svg>
            )}
            <span className="text-x-text-secondary truncate">
              @{post.actor.handle}
            </span>
            <span className="text-x-text-secondary">·</span>
            <span className="text-x-text-secondary hover:underline">
              {formatRelativeTime(post.createdAt)}
            </span>
            {post.actor.kind === "agent" && post.trigger === "followup" && (
              <span className="ml-auto rounded-full border border-x-border px-2 py-0.5 text-[11px] font-medium text-x-text-secondary dark:border-x-border-dark">
                followup
              </span>
            )}
            {onDelete && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  void handleDelete();
                }}
                className="pointer-events-auto ml-auto rounded-full px-2 py-1 text-xs font-semibold text-x-text-secondary transition-colors hover:bg-red-500/10 hover:text-red-600 dark:hover:text-red-400"
                aria-label="Delete post"
              >
                Delete
              </button>
            )}
          </div>

          {/* 专业领域 */}
          {post.actor.specialty && (
            <p className="text-xs text-x-text-secondary mt-0.5">{post.actor.specialty}</p>
          )}

          {/* 回复提示 */}
            {post.kind === "reply" && post.referencedPost && (
            <p className="mt-1 text-sm text-x-text-secondary">
              Replying to{" "}
              <Link
                to={`/profile/${post.referencedPost.actor.handle}`}
                className="pointer-events-auto text-x-primary hover:underline"
                onClick={(e) => e.stopPropagation()}
              >
                @{post.referencedPost.actor.handle}
              </Link>
            </p>
          )}

          {/* 帖子正文 */}
          {post.quoteBody && (
            <MarkdownText
              content={post.quoteBody}
              className="mt-2 text-[15px] leading-normal"
            />
          )}
          {post.body && (
            <MarkdownText
              content={post.body}
              className="mt-2 text-[15px] leading-normal"
            />
          )}

          {/* 图片附件 */}
          <PostMediaGrid media={post.media ?? []} />

          {/* 文件附件 */}
          {post.files?.map((file) => (
            <FileCard key={file.id} file={file} />
          ))}

          {/* 引用的帖子 */}
          {post.referencedPost && <ReferencedPostPreview referencedPost={post.referencedPost} />}

          {/* 互动按钮 */}
          <div
            className="pointer-events-auto mt-3 flex max-w-md items-center justify-between"
            onClick={(e) => e.stopPropagation()}
          >
            {/* 回复 */}
            <ReplyButton count={post.replyCount} onReply={handleOpenReply} />

            {/* 转发 */}
            <RepostButton
              reposted={false}
              count={post.repostCount}
              onRepost={handleRepost}
            />

            {/* 点赞 */}
            <LikeButton
              liked={post.likedByYou}
              count={post.likeCount}
              onLike={handleLike}
            />

            {/* 置顶 */}
            {onPin && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onPin(post.id);
                }}
                className={cn(
                  "group flex items-center gap-1.5 text-sm transition-colors",
                  post.pinnedAt != null
                    ? "text-x-primary"
                    : "text-x-text-secondary hover:text-x-primary"
                )}
              >
                <div className={cn(
                  "flex h-9 w-9 items-center justify-center rounded-full transition-colors",
                  post.pinnedAt != null
                    ? "bg-x-primary/10"
                    : "group-hover:bg-x-primary/10"
                )}>
                  <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
                    <path d="M7 4.5C7 3.12 8.12 2 9.5 2h5C15.88 2 17 3.12 17 4.5v5.26L20 16h-7v5l-1 2-1-2v-5H4l3-6.26V4.5z" />
                  </svg>
                </div>
              </button>
            )}
          </div>
        </div>
      </div>
    </motion.article>
  );
}

export const PostCard = memo(
  PostCardComponent,
  (prev, next) =>
    prev.post === next.post &&
    prev.animated === next.animated &&
    prev.onLike === next.onLike &&
    prev.onRepost === next.onRepost &&
    prev.onDelete === next.onDelete &&
    prev.onPin === next.onPin
);
