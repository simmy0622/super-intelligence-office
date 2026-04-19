import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { type FeedPost } from "../lib/client";

interface StandupCardProps {
  post: FeedPost;
}

export function StandupCard({ post }: StandupCardProps) {
  const [expanded, setExpanded] = useState(false);

  if (!post.body) return null;

  let standupData = null;
  const isStandup = post.body.trim().startsWith("<standup>");

  if (isStandup) {
    try {
      const jsonStr = post.body.replace(/^<standup>/, "").replace(/<\/standup>$/, "");
      standupData = JSON.parse(jsonStr);
    } catch (e) {
      console.error("Failed to parse standup JSON", e);
    }
  }

  // Fallback to normal text if not parsed as standup JSON
  if (!standupData) {
    return (
      <div className="border-b border-x-border dark:border-x-border-dark px-4 py-3 bg-x-surface-hover/30 dark:bg-x-surface-hover-dark/30">
        <div className="whitespace-pre-wrap text-sm text-x-text dark:text-x-text-dark">
          {post.body}
        </div>
      </div>
    );
  }

  const dateStr = new Date(post.createdAt * 1000).toLocaleDateString("zh-CN", {
    weekday: 'short',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  });

  return (
    <div 
      className="border-b border-x-border dark:border-x-border-dark bg-x-surface-hover/30 dark:bg-x-surface-hover-dark/30 hover:bg-x-surface-hover/50 dark:hover:bg-x-surface-hover-dark/50 transition-colors cursor-pointer group"
      onClick={() => setExpanded(!expanded)}
    >
      <div className="px-4 py-3">
        {/* Pinned Indicator */}
        <div className="mb-1 flex items-center gap-2 pl-[44px] text-xs font-bold text-x-text-secondary">
          <svg viewBox="0 0 24 24" className="h-3.5 w-3.5" fill="currentColor">
            <path d="M7 4.5C7 3.12 8.12 2 9.5 2h5C15.88 2 17 3.12 17 4.5v5.26L20 16h-7v5l-1 2-1-2v-5H4l3-6.26V4.5z" />
          </svg>
          <span>Pinned Standup</span>
        </div>

        <div className="flex gap-3">
          {/* Avatar / Icon Column */}
          <div className="flex flex-col items-center shrink-0">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-x-primary/10 text-x-primary dark:bg-x-primary/20">
              <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
                <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
              </svg>
            </div>
            <div className="mt-2 w-0.5 flex-1 bg-x-border dark:bg-x-border-dark opacity-50" />
          </div>

          {/* Content Column */}
          <div className="min-w-0 flex-1 pb-2">
            {/* Header */}
            <div className="flex items-center gap-1.5 text-[15px]">
              <span className="font-bold text-x-text dark:text-x-text-dark">Daily Standup</span>
              <svg viewBox="0 0 24 24" className="h-4 w-4 text-x-primary" fill="currentColor">
                <path d="M22.25 12c0-1.43-.88-2.67-2.19-3.34.46-1.39.2-2.9-.81-3.91s-2.52-1.27-3.91-.81c-.66-1.31-1.91-2.19-3.34-2.19s-2.67.88-3.33 2.19c-1.4-.46-2.91-.2-3.92.81s-1.26 2.52-.8 3.91c-1.31.67-2.2 1.91-2.2 3.34s.89 2.67 2.2 3.34c-.46 1.39-.21 2.9.8 3.91s2.52 1.26 3.91.81c.67 1.31 1.91 2.19 3.34 2.19s2.68-.88 3.34-2.19c1.39.45 2.9.2 3.91-.81s1.27-2.52.81-3.91c1.31-.67 2.19-1.91 2.19-3.34zm-11.71 4.2L6.8 12.46l1.41-1.42 2.26 2.26 4.8-5.23 1.47 1.36-6.2 6.77z" />
              </svg>
              <span className="text-x-text-secondary">@Nomi</span>
              <span className="text-x-text-secondary">·</span>
              <span className="text-x-text-secondary hover:underline">{dateStr}</span>
            </div>

            <p className="mt-1 mb-4 text-[15px] font-medium text-x-text dark:text-x-text-dark leading-normal">
              "{standupData.intro}"
            </p>

            {/* Agent Items */}
            <div className="space-y-4">
              {standupData.agents?.map((agent: any, idx: number) => (
                <div key={idx} className="flex flex-col text-[15px]">
                  <div className="flex items-center gap-2 mb-1.5">
                    <span className="font-bold text-x-text dark:text-x-text-dark">
                      {agent.handle}
                    </span>
                    <div className="h-px flex-1 bg-x-border dark:bg-x-border-dark opacity-50"></div>
                  </div>
                  <div className="pl-1 leading-normal">
                    <div className="text-x-text-secondary">
                      <span className="font-semibold text-x-text dark:text-x-text-dark">昨天：</span>{agent.yesterday}
                    </div>
                    
                    <AnimatePresence>
                      {expanded && (
                        <motion.div
                          initial={{ height: 0, opacity: 0 }}
                          animate={{ height: "auto", opacity: 1 }}
                          exit={{ height: 0, opacity: 0 }}
                          className="overflow-hidden"
                        >
                          <div className="mt-1.5 text-x-primary dark:text-blue-400">
                            <span className="font-semibold">今日：</span>{agent.focus}
                          </div>
                          {agent.blocker && (
                            <div className="mt-1.5 font-semibold text-red-500 dark:text-red-400">
                              ⚠️ 阻碍: {agent.blocker}
                            </div>
                          )}
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                </div>
              ))}
            </div>

            <p className="mt-5 mb-2 text-[15px] italic text-x-text-secondary text-right">
              "{standupData.closing}"
            </p>

            <div className="mt-3 flex justify-center">
              <span className="text-[13px] font-bold text-x-primary group-hover:underline">
                {expanded ? "Show less" : "Show more"}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}