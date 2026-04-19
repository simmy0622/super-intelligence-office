import { motion } from "framer-motion";
import type { ReactNode } from "react";
import type { FeedPost } from "../lib/client";
import { PostCard } from "./PostCard";

interface PostListProps {
  posts: FeedPost[];
  emptyTitle?: string;
  emptyDescription?: string;
  onLike?: (postId: number) => Promise<void> | void;
  onRepost?: (postId: number) => Promise<void> | void;
  onDelete?: (postId: number) => Promise<void>;
  onPin?: (postId: number) => void;
  renderExtra?: (post: FeedPost) => ReactNode;
}

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  show: { 
    opacity: 1, 
    y: 0,
    transition: {
      duration: 0.3,
      ease: "easeOut" as const,
    },
  },
};

export function PostList({
  posts,
  emptyTitle = "No posts yet",
  emptyDescription = "The timeline is empty. Be the first to post!",
  onLike,
  onRepost,
  onDelete,
  onPin,
  renderExtra,
}: PostListProps) {
  const useEntranceAnimation = posts.length <= 40;

  if (posts.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center px-8 py-16 text-center">
        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-x-surface-hover dark:bg-x-surface-hover-dark">
          <svg
            viewBox="0 0 24 24"
            className="h-8 w-8 text-x-text-secondary"
            fill="currentColor"
          >
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z" />
          </svg>
        </div>
        <h3 className="text-2xl font-bold text-x-text dark:text-x-text-dark">{emptyTitle}</h3>
        <p className="mt-2 max-w-sm text-x-text-secondary">{emptyDescription}</p>
      </div>
    );
  }

  if (!useEntranceAnimation) {
    return (
      <div className="divide-y divide-x-border dark:divide-x-border-dark">
        {posts.map((post) => (
          <div key={post.id}>
            <PostCard
              post={post}
              onLike={onLike}
              onRepost={onRepost}
              onDelete={onDelete}
              onPin={onPin}
              animated={false}
            />
            {renderExtra?.(post)}
          </div>
        ))}
      </div>
    );
  }

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="divide-y divide-x-border dark:divide-x-border-dark"
    >
      {posts.map((post) => (
        <motion.div key={post.id} variants={itemVariants}>
          <PostCard
            post={post}
            onLike={onLike}
            onRepost={onRepost}
            onDelete={onDelete}
            onPin={onPin}
            animated
          />
          {renderExtra?.(post)}
        </motion.div>
      ))}
    </motion.div>
  );
}
