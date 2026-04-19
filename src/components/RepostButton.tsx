import { useState } from "react";
import { motion } from "framer-motion";
import { cn } from "../lib/utils";

interface RepostButtonProps {
  reposted: boolean;
  count: number;
  onRepost: () => void;
}

export function RepostButton({ reposted, count, onRepost }: RepostButtonProps) {
  const [isAnimating, setIsAnimating] = useState(false);

  const handleClick = () => {
    setIsAnimating(true);
    setTimeout(() => setIsAnimating(false), 300);
    onRepost();
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      className={cn(
        "group flex items-center gap-1.5 text-sm transition-colors",
        reposted ? "text-x-repost" : "text-x-text-secondary hover:text-x-repost"
      )}
    >
      <div
        className={cn(
          "flex h-9 w-9 items-center justify-center rounded-full transition-colors",
          reposted ? "bg-x-repost-hover" : "group-hover:bg-x-repost-hover"
        )}
      >
        <motion.svg
          viewBox="0 0 24 24"
          className="h-5 w-5"
          animate={isAnimating ? { rotate: [0, -20, 20, 0], scale: [1, 0.9, 1.1, 1] } : {}}
          transition={{ duration: 0.4, ease: "easeInOut" }}
        >
          <path
            d="M4.5 3.88l4.432 4.14-1.364 1.46L5.5 7.55V16c0 1.1.896 2 2 2H13v2H7.5c-2.209 0-4-1.79-4-4V7.55L1.432 9.48.068 8.02 4.5 3.88zM16.5 6H11V4h5.5c2.209 0 4 1.79 4 4v8.45l2.068-1.93 1.364 1.46-4.432 4.14-4.432-4.14 1.364-1.46 2.068 1.93V8c0-1.1-.896-2-2-2z"
            fill="currentColor"
          />
        </motion.svg>
      </div>
      <motion.span
        key={count}
        initial={{ y: reposted ? -10 : 10, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ type: "spring", stiffness: 500, damping: 30 }}
      >
        {count > 0 ? count : ""}
      </motion.span>
    </button>
  );
}
