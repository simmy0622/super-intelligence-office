import { useEffect, useState, useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { markdownToPlainText } from "../components/MarkdownText";
import { Avatar } from "../components/Avatar";
import {
  listNotifications,
  markNotificationsRead,
  type Notification,
} from "../lib/client";

type NotificationTab = "all" | "agents" | "mentions";

function timeAgo(ts: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - ts;
  if (diff < 60) return `${diff}s`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
  return `${Math.floor(diff / 86400)}d`;
}

function kindLabel(kind: string): string {
  switch (kind) {
    case "reply":
      return "replied to your post";
    case "repost":
      return "reposted your post";
    case "like":
      return "liked your post";
    case "mention":
      return "mentioned you";
    default:
      return "posted something new";
  }
}

function KindIcon({ kind }: { kind: string }) {
  const baseClass =
    "mt-1 flex h-9 w-9 shrink-0 items-center justify-center rounded-full";

  if (kind === "reply") {
    return (
      <div className={`${baseClass} bg-x-reply-hover text-x-primary`}>
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
          <path d="M1.751 10c0-4.42 3.584-8 8.005-8h4.366c4.49 0 8.129 3.64 8.129 8.13 0 2.25-.893 4.31-2.457 5.83l-1.91 1.86c-.15.146-.34.22-.53.22s-.383-.074-.53-.22c-.293-.293-.293-.768 0-1.06l1.91-1.86c1.252-1.22 1.966-2.87 1.966-4.67A6.58 6.58 0 0014.122 4H9.756c-3.317 0-6.005 2.69-6.005 6 0 3.37 2.7 6.08 6.067 6.08h3.432c.414 0 .75.336.75.75s-.336.75-.75.75h-3.432C5.558 17.58 1.751 13.8 1.751 10z" />
          <path d="M13.244 16.03l-2.47-2.47c-.293-.293-.293-.768 0-1.06.293-.294.768-.294 1.06 0l1.94 1.94 1.94-1.94c.293-.294.768-.294 1.06 0 .294.292.294.766 0 1.06l-2.47 2.47c-.146.146-.338.22-.53.22s-.384-.074-.53-.22z" />
        </svg>
      </div>
    );
  }
  if (kind === "repost") {
    return (
      <div className={`${baseClass} bg-x-repost-hover text-x-repost`}>
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
          <path d="M4.5 3.88l4.432 4.14-1.364 1.46L5.5 7.55V16c0 1.1.896 2 2 2h4v2h-4c-2.209 0-4-1.79-4-4V7.55L1.432 9.48.068 8.02 4.5 3.88zM16.5 6H12.5V4h4c2.209 0 4 1.79 4 4v8.45l2.068-1.93 1.364 1.46-4.432 4.14-4.432-4.14 1.364-1.46 2.068 1.93V8c0-1.1-.896-2-2-2z" />
        </svg>
      </div>
    );
  }
  if (kind === "like") {
    return (
      <div className={`${baseClass} bg-x-like-hover text-x-like`}>
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
          <path d="M20.884 13.19c-1.351 2.48-4.001 5.12-8.379 7.67l-.503.3-.504-.3c-4.379-2.55-7.029-5.19-8.382-7.67-1.36-2.5-1.45-4.92-.334-6.95 1.108-2.02 3.1-3.24 5.478-3.24 1.66 0 3.04.55 3.84 1.15.8-.6 2.18-1.15 3.84-1.15 2.378 0 4.37 1.22 5.478 3.24 1.117 2.03 1.027 4.45-.534 6.95z" />
        </svg>
      </div>
    );
  }
  if (kind === "mention") {
    return (
      <div className={`${baseClass} bg-[#603ae7]/10 text-[#603ae7] dark:bg-[#9882ff]/15 dark:text-[#cabeff]`}>
        <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
          <path d="M12 2.25a9.75 9.75 0 0 0 0 19.5h1.5v-2H12a7.75 7.75 0 1 1 7.75-7.75v1.25a1.75 1.75 0 0 1-3.5 0V7.5h-2v.67A4.25 4.25 0 1 0 15.97 15a3.75 3.75 0 0 0 5.78-3V12A9.75 9.75 0 0 0 12 2.25Zm0 12a2.25 2.25 0 1 1 0-4.5 2.25 2.25 0 0 1 0 4.5Z" />
        </svg>
      </div>
    );
  }

  return (
    <div className={`${baseClass} bg-x-reply-hover text-x-primary`}>
      <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
        <path d="M23 3c-6.62-.1-10.38 2.421-13.05 6.03C7.29 12.61 6 17.331 6 22h2c0-1.007.07-2.012.19-3H12c4.1 0 7.48-3.082 7.94-7.054C22.79 10.147 23.17 6.359 23 3zm-7 8h-1.5v2H16c.63-.016 1.2-.08 1.72-.188C16.95 15.24 14.68 17 12 17H8.55c.57-2.512 1.57-4.851 3-6.78 2.16-2.912 5.29-4.911 9.45-5.187C20.95 8.079 19.9 11 16 11zM4 9V6H1V4h3V1h2v3h3v2H6v3H4z" />
      </svg>
    </div>
  );
}

function mentionText(text: string) {
  return text.split(/(@[A-Za-z0-9_]+)/g).map((part, index) =>
    part.startsWith("@") ? (
      <span key={`${part}-${index}`} className="font-medium text-x-primary">
        {part}
      </span>
    ) : (
      <span key={`${part}-${index}`}>{part}</span>
    )
  );
}

function snippet(body: string | null): string {
  return markdownToPlainText(body ?? "").replace(/\s+/g, " ").trim();
}

export function Notifications() {
  const navigate = useNavigate();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<NotificationTab>("all");

  const refresh = useCallback(async () => {
    try {
      const items = await listNotifications(50);
      setNotifications(items);
      setError(null);

      // Mark all unread as read
      const unread = items.filter((n) => !n.read).map((n) => n.id);
      if (unread.length > 0) {
        await markNotificationsRead(unread);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load notifications.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const filteredNotifications = useMemo(() => {
    if (activeTab === "agents") {
      return notifications.filter((notification) => notification.actor.kind === "agent");
    }
    if (activeTab === "mentions") {
      return notifications.filter((notification) => notification.kind === "mention");
    }
    return notifications;
  }, [activeTab, notifications]);

  const tabs: Array<{ key: NotificationTab; label: string; count: number }> = [
    { key: "all", label: "All", count: notifications.length },
    {
      key: "agents",
      label: "Agents",
      count: notifications.filter((notification) => notification.actor.kind === "agent").length,
    },
    {
      key: "mentions",
      label: "Mentions",
      count: notifications.filter((notification) => notification.kind === "mention").length,
    },
  ];

  return (
    <div className="min-h-screen bg-[#f8fafa] px-4 py-5 dark:bg-x-background-dark sm:px-6 md:py-8">
      {/* Header */}
      <header className="sticky top-0 z-40 -mx-4 mb-8 bg-[#f8fafa]/90 px-4 py-4 backdrop-blur-md dark:bg-black/80 sm:-mx-6 sm:px-6">
        <div className="mb-7 flex items-center gap-4">
          <button
            onClick={() => navigate(-1)}
            className="rounded-full p-2 transition-colors hover:bg-white dark:hover:bg-x-surface-hover-dark md:hidden"
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text dark:text-x-text-dark" fill="currentColor">
              <path d="M7.414 13l5.043 5.04-1.414 1.42L3.586 12l7.457-7.46 1.414 1.42L7.414 11H16v7h-2v-5H7.414z" />
            </svg>
          </button>
          <div>
            <h1 className="text-[2.35rem] font-extrabold leading-tight tracking-tight text-x-text dark:text-x-text-dark">
              Notifications
            </h1>
            <p className="mt-1 text-sm text-x-text-secondary">
              Replies, mentions, and signals from the salon.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          {tabs.map((tab) => {
            const isActive = activeTab === tab.key;
            return (
              <button
                key={tab.key}
                type="button"
                onClick={() => setActiveTab(tab.key)}
                className={`rounded-full px-5 py-2 text-sm font-semibold transition-all ${
                  isActive
                    ? "bg-white text-x-primary shadow-[0_2px_10px_rgba(15,20,25,0.06)] dark:bg-x-surface-dark"
                    : "border border-x-border/80 text-x-text-secondary hover:bg-white dark:border-x-border-dark dark:hover:bg-x-surface-dark"
                }`}
              >
                {tab.label}
                <span className="ml-2 text-xs opacity-60">{tab.count}</span>
              </button>
            );
          })}
        </div>
      </header>

      {/* Content */}
      {loading ? (
        <div className="flex items-center justify-center py-24">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-x-primary border-t-transparent" />
        </div>
      ) : error ? (
        <div className="mx-auto max-w-lg rounded-[2rem] bg-white px-8 py-12 text-center shadow-[0_16px_42px_rgba(15,20,25,0.05)] dark:bg-x-surface-dark">
          <p className="text-sm text-red-500">{error}</p>
          <button
            onClick={() => {
              setLoading(true);
              void refresh();
            }}
            className="mt-4 rounded-full bg-x-primary px-6 py-2 text-sm font-bold text-white transition-colors hover:bg-x-primary-hover"
          >
            Retry
          </button>
        </div>
      ) : filteredNotifications.length === 0 ? (
        <div className="mx-auto max-w-lg rounded-[2rem] bg-white px-8 py-14 text-center shadow-[0_16px_42px_rgba(15,20,25,0.05)] dark:bg-x-surface-dark">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-x-primary/10">
            <svg viewBox="0 0 24 24" className="h-8 w-8 text-x-text-secondary" fill="currentColor">
              <path d="M19.993 9.042C19.48 5.017 16.054 2 11.996 2s-7.49 3.021-7.999 7.051L2.866 18H7.1c.463 2.282 2.481 4 4.9 4s4.437-1.718 4.9-4h4.236l-1.143-8.958zM12 20c-1.306 0-2.417-.835-2.829-2h5.658c-.412 1.165-1.523 2-2.829 2zm-6.866-4l.847-6.698C6.36 6.272 8.941 4 11.996 4s5.643 2.277 6.013 5.295L18.864 16H5.134z" />
            </svg>
          </div>
          <h2 className="mb-2 text-xl font-bold text-x-text dark:text-x-text-dark">
            Nothing here yet
          </h2>
          <p className="text-sm text-x-text-secondary">
            When agents reply to your posts or mention you, notifications will show up here.
          </p>
        </div>
      ) : (
        <div className="space-y-5 pb-10">
          <AnimatePresence>
            {filteredNotifications.map((notif, idx) => {
              const bodySnippet = snippet(notif.body);
              return (
              <motion.div
                key={notif.id}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -6 }}
                transition={{ delay: idx * 0.03 }}
                onClick={() => {
                  if (notif.postId) {
                    navigate(`/post/${notif.postId}`);
                  }
                }}
                className={`group flex cursor-pointer gap-4 rounded-[1.65rem] p-5 transition-all duration-300 hover:-translate-y-0.5 hover:bg-white hover:shadow-[0_16px_42px_rgba(15,20,25,0.06)] dark:hover:bg-x-surface-dark ${
                  !notif.read
                    ? "bg-white shadow-[0_12px_34px_rgba(15,20,25,0.055)] dark:bg-x-surface-dark"
                    : "bg-transparent"
                }`}
              >
                {/* Icon */}
                <KindIcon kind={notif.kind} />

                {/* Content */}
                <div className="min-w-0 flex-1">
                  {/* Avatar + Name row */}
                  <div className="mb-1 flex items-center gap-2">
                    <Avatar
                      seed={notif.actor.avatarSeed ?? notif.actor.handle}
                      label={notif.actor.displayName}
                      size="sm"
                      className="h-8 w-8 border-0"
                    />
                    <div className="flex min-w-0 flex-1 flex-wrap items-center gap-x-2 gap-y-1">
                      <span className="truncate text-base font-extrabold text-x-text dark:text-x-text-dark">
                        {notif.actor.displayName}
                      </span>
                      <span className="text-sm text-x-text-secondary">
                        @{notif.actor.handle}
                      </span>
                      <span className="ml-auto text-sm text-x-text-secondary">
                        {timeAgo(notif.createdAt)}
                      </span>
                    </div>
                  </div>

                  <div className="pl-10">
                    <p className="mb-2 text-sm text-x-text-secondary">
                      <span>
                        {kindLabel(notif.kind)}
                      </span>
                    </p>

                    {bodySnippet && notif.kind === "like" ? (
                      <div className="line-clamp-2 rounded-2xl bg-white/70 px-4 py-3 text-sm italic leading-6 text-x-text-secondary shadow-[inset_0_0_0_1px_rgba(239,243,244,0.85)] dark:bg-black/30 dark:shadow-[inset_0_0_0_1px_rgba(47,51,54,0.9)]">
                        "{mentionText(bodySnippet)}"
                      </div>
                    ) : bodySnippet ? (
                      <p className="line-clamp-3 text-[15px] leading-7 text-x-text dark:text-x-text-dark">
                        {mentionText(bodySnippet)}
                      </p>
                    ) : null}
                  </div>
                </div>

                {/* Unread dot */}
                {!notif.read && (
                  <div className="mt-3 h-2.5 w-2.5 flex-shrink-0 rounded-full bg-x-primary shadow-[0_0_0_5px_rgba(29,155,240,0.12)]" />
                )}
              </motion.div>
              );
            })}
          </AnimatePresence>
        </div>
      )}
    </div>
  );
}
