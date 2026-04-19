import { useEffect, useMemo, useRef, useState, useCallback } from "react";
import { useParams } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import { Avatar } from "../components/Avatar";
import { PostList } from "../components/PostList";
import { QuoteComposerModal } from "../components/QuoteComposerModal";
import { EditProfileModal } from "../components/EditProfileModal";
import { updatePost, upsertPost } from "../lib/post-state";
import {
  DEFAULT_AGENT_TOOLS,
  deletePost,
  getActor,
  getAgentDisabledDefaultTools,
  likeToggle,
  listPosts,
  repostAsHuman,
  runAgentStep,
  saveAgentDisabledDefaultTools,
  saveProfile,
  getProfile,
  type Actor,
  type FeedPost,
  type UserProfile,
} from "../lib/client";
import { useLanguage } from "../lib/language";
import { cn } from "../lib/utils";
import { useSalon } from "../lib/salon-context";

export function Profile() {
  const { handle = "" } = useParams();
  const { activeSalonId } = useSalon();
  const { t } = useLanguage();
  const [actor, setActor] = useState<Actor | null>(null);
  const [posts, setPosts] = useState<FeedPost[]>([]);
  const postsRef = useRef<FeedPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [repostTarget, setRepostTarget] = useState<FeedPost | null>(null);
  const [reposting, setReposting] = useState(false);
  const [activeTab, setActiveTab] = useState<"posts" | "replies" | "media">("posts");

  const [searchingImage, setSearchingImage] = useState<"banner" | null>(null);
  const [imageMessage, setImageMessage] = useState<string | null>(null);
  const [imageError, setImageError] = useState<string | null>(null);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showAgentAvatarModal, setShowAgentAvatarModal] = useState(false);
  const [showToolboxModal, setShowToolboxModal] = useState(false);
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);
  const [disabledDefaultTools, setDisabledDefaultTools] = useState<string[]>([]);
  const [savingToolName, setSavingToolName] = useState<string | null>(null);
  const [toolboxError, setToolboxError] = useState<string | null>(null);

  const loadProfile = useCallback(async () => {
    setLoading(true);
    try {
      const [nextActor, allPosts, profile, nextDisabledTools] = await Promise.all([
        getActor(handle),
        listPosts(undefined, 100, activeSalonId),
        getProfile(handle),
        getAgentDisabledDefaultTools(handle).catch(() => []),
      ]);
      setActor(nextActor);
      setPosts(allPosts);
      setUserProfile(profile);
      setDisabledDefaultTools(nextActor.kind === "agent" ? nextDisabledTools : []);
      setError(null);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to load profile.");
    } finally {
      setLoading(false);
    }
  }, [handle, activeSalonId]);

  useEffect(() => {
    void loadProfile();
  }, [loadProfile]);

  useEffect(() => {
    setShowToolboxModal(false);
    setShowAgentAvatarModal(false);
    setToolboxError(null);
  }, [handle]);

  useEffect(() => {
    postsRef.current = posts;
  }, [posts]);

  const actorPosts = useMemo(
    () => posts.filter((post) => post.actor.handle.toLowerCase() === handle.toLowerCase()),
    [handle, posts]
  );
  
  const visiblePosts = useMemo(() => {
    if (activeTab === "posts") {
      return actorPosts.filter((post) => post.kind !== "reply");
    }
    if (activeTab === "replies") {
      return actorPosts.filter((post) => post.kind === "reply");
    }
    return actorPosts;
  }, [activeTab, actorPosts]);

  const enabledDefaultToolCount = useMemo(
    () => DEFAULT_AGENT_TOOLS.filter((tool) => !disabledDefaultTools.includes(tool.name)).length,
    [disabledDefaultTools]
  );

  const handleLike = useCallback((postId: number) => {
    setPosts((current) =>
      updatePost(current, postId, (post) => ({
        ...post,
        likedByYou: !post.likedByYou,
        likeCount: Math.max(0, post.likeCount + (post.likedByYou ? -1 : 1)),
      }))
    );
    likeToggle(postId).catch(() => {
      setPosts((current) =>
        updatePost(current, postId, (post) => ({
          ...post,
          likedByYou: !post.likedByYou,
          likeCount: Math.max(0, post.likeCount + (post.likedByYou ? -1 : 1)),
        }))
      );
    });
  }, []);

  const handleRepost = useCallback((postId: number) => {
    setRepostTarget(postsRef.current.find((post) => post.id === postId) ?? null);
  }, []);

  const handleDelete = useCallback(async (postId: number) => {
    const result = await deletePost(postId);
    const deleted = new Set(result.deletedPostIds);
    setPosts((current) => current.filter((post) => !deleted.has(post.id)));
    setError(null);
  }, []);

  const handleSubmitRepost = async (quoteBody: string | null) => {
    if (!repostTarget) return;

    setReposting(true);
    try {
      const reposted = await repostAsHuman(repostTarget.id, quoteBody, repostTarget.salonId);
      const alreadyVisible = posts.some((post) => post.id === reposted.id);
      setPosts((current) => upsertPost(current, reposted));
      if (!alreadyVisible) {
        setPosts((current) =>
          updatePost(current, repostTarget.id, (post) => ({
            ...post,
            repostCount: post.repostCount + 1,
          }))
        );
      }
      setRepostTarget(null);
    } finally {
      setReposting(false);
    }
  };


  const handleAutoBanner = async () => {
    if (!actor || actor.kind !== "agent") return;
    setSearchingImage("banner");
    setImageMessage(null);
    setImageError(null);
    try {
      const result = await runAgentStep(actor.handle, "find_banner", null, null);
      const raw = result.assistantContent ?? "";
      const urlMatch = raw.match(/https?:\/\/[^\s"'<>]+/);
      const url = urlMatch?.[0]?.replace(/[.,;!?)]+$/, "") ?? "";
      if (!url) {
        setImageError(`@${actor.handle} couldn't find a banner image. Try again.`);
        return;
      }
      const current = userProfile ?? { handle, displayName: displayName, bio: displayBio };
      await saveProfile(handle, {
        displayName: current.displayName,
        bio: current.bio ?? "",
        avatar: current.avatar ?? displayAvatar ?? undefined,
        banner: url,
      });
      setUserProfile((prev) => ({
        handle: prev?.handle ?? handle,
        displayName: prev?.displayName ?? displayName,
        bio: prev?.bio ?? displayBio,
        avatar: prev?.avatar,
        banner: url,
      }));
      setImageMessage(`✦ Banner updated by @${actor.handle}`);
    } catch (err) {
      setImageError(formatBannerError(err));
    } finally {
      setSearchingImage(null);
    }
  };

  const formatBannerError = (err: unknown) => {
    const message = err instanceof Error ? err.message : "Failed to find banner image.";
    if (
      message.includes("api.deepseek.com") ||
      message.includes("api.tavily.com") ||
      message.includes("error sending request")
    ) {
      return "External image search is unreachable right now. Check network/VPN or API access, then try again.";
    }
    return message;
  };

  const handleSaveProfile = async (data: {
    avatar?: string;
    banner?: string;
    displayName: string;
    bio: string;
  }) => {
    await saveProfile(handle, data);
    setUserProfile((prev) => ({
      handle: prev?.handle || handle,
      avatar: data.avatar,
      banner: data.banner,
      displayName: data.displayName,
      bio: data.bio,
    }));
    // Reload to get updated actor data
    await loadProfile();
  };

  const handleToggleDefaultTool = async (toolName: string) => {
    const wasDisabled = disabledDefaultTools.includes(toolName);
    const nextDisabledTools = wasDisabled
      ? disabledDefaultTools.filter((name) => name !== toolName)
      : [...disabledDefaultTools, toolName];

    setDisabledDefaultTools(nextDisabledTools);
    setSavingToolName(toolName);
    setToolboxError(null);
    try {
      const saved = await saveAgentDisabledDefaultTools(handle, nextDisabledTools);
      setDisabledDefaultTools(saved);
    } catch (nextError) {
      setDisabledDefaultTools(disabledDefaultTools);
      setToolboxError(
        nextError instanceof Error ? nextError.message : "Failed to update toolbox settings.",
      );
    } finally {
      setSavingToolName(null);
    }
  };

  const isOwnProfile = actor?.kind === "human";
  const displayAvatar = userProfile?.avatar || actor?.avatarSeed;
  const displayBanner = userProfile?.banner;
  const displayName = userProfile?.displayName || actor?.displayName || handle;
  const displayBio = userProfile?.bio ?? actor?.bio ?? "";

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-x-primary border-t-transparent" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-screen flex-col items-center justify-center px-8">
        <p className="text-red-500">{error}</p>
        <button
          onClick={() => void loadProfile()}
          className="mt-4 rounded-full bg-x-primary px-6 py-2 text-sm font-bold text-white hover:bg-x-primary-hover"
        >
          {t("common.retry")}
        </button>
      </div>
    );
  }

  if (!actor) {
    return (
      <div className="flex h-screen items-center justify-center">
        <p className="text-x-text-secondary">{t("profile.profileNotFound")}</p>
      </div>
    );
  }

  return (
    <div>
      {/* Header */}
      <header className="sticky top-0 z-50 glass border-b border-x-border dark:border-x-border-dark">
        <div className="flex h-14 items-center gap-4 px-4">
          <button
            onClick={() => window.history.back()}
            className="rounded-full p-2 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text dark:text-x-text-dark" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <path d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <div>
            <h1 className="text-xl font-bold text-x-text dark:text-x-text-dark">{displayName}</h1>
            <p className="text-sm text-x-text-secondary">{t("profile.posts", { count: actorPosts.length })}</p>
          </div>
        </div>
      </header>

      {/* Profile Header */}
      <div className="relative">
        {/* Banner */}
        <div className={cn(
          "h-48",
          !displayBanner && "bg-gradient-to-r from-x-primary via-purple-500 to-pink-500"
        )}>
          {displayBanner && (
            <img
              src={displayBanner}
              alt="Banner"
              className="h-full w-full object-cover"
            />
          )}
        </div>

        {/* Profile Info */}
        <div className="px-4 pb-4">
          {/* Avatar & Actions */}
          <div className="relative -mt-16 mb-4 flex items-end justify-between">
            <div className="rounded-full border-4 border-x-background dark:border-x-background-dark bg-x-background dark:bg-x-background-dark overflow-hidden">
              {displayAvatar?.startsWith("data:") ? (
                <img
                  src={displayAvatar}
                  alt={displayName}
                  className="h-20 w-20 object-cover"
                />
              ) : (
                <Avatar seed={displayAvatar} label={displayName} size="lg" />
              )}
            </div>
            <div className="mb-2">
              {isOwnProfile ? (
                <button
                  onClick={() => setShowEditModal(true)}
                  className="rounded-full border border-x-border dark:border-x-border-dark px-5 py-2 text-sm font-bold text-x-text dark:text-x-text-dark transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
                >
                  {t("profile.editProfile")}
                </button>
              ) : (
                <div className="flex flex-col items-end gap-1.5">
                  <div className="flex items-center gap-2">
                    {actor.kind === "agent" && (
                      <button
                        type="button"
                        onClick={() => setShowAgentAvatarModal(true)}
                        className="flex items-center gap-1.5 rounded-full bg-x-primary px-3 py-1.5 text-xs font-bold text-white shadow-sm transition-colors hover:bg-x-primary-hover"
                      >
                        <svg viewBox="0 0 24 24" className="h-3.5 w-3.5" fill="currentColor">
                          <path d="M12 12c2.761 0 5-2.239 5-5S14.761 2 12 2 7 4.239 7 7s2.239 5 5 5zm0 2c-3.866 0-7 2.239-7 5v1h14v-1c0-2.761-3.134-5-7-5z" />
                        </svg>
                        {t("profile.editAvatar")}
                      </button>
                    )}
                    {actor.kind === "agent" && (
                      <button
                        type="button"
                        onClick={() => setShowToolboxModal(true)}
                        className="flex items-center gap-1.5 rounded-full bg-x-primary px-3 py-1.5 text-xs font-bold text-white shadow-sm transition-colors hover:bg-x-primary-hover"
                      >
                        <svg viewBox="0 0 24 24" className="h-3.5 w-3.5" fill="currentColor">
                          <path d="M21 7.5V6a2 2 0 0 0-2-2h-3.18A3 3 0 0 0 13 2h-2a3 3 0 0 0-2.82 2H5a2 2 0 0 0-2 2v1.5A2.5 2.5 0 0 0 1.5 10v2A2.5 2.5 0 0 0 3 14.5V18a2 2 0 0 0 2 2h4v-6h6v6h4a2 2 0 0 0 2-2v-3.5a2.5 2.5 0 0 0 1.5-2.5v-2A2.5 2.5 0 0 0 21 7.5zM10 5a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v1h-4V5zm10.5 7a.5.5 0 0 1-.5.5H15v-2H9v2H4a.5.5 0 0 1-.5-.5v-2A.5.5 0 0 1 4 9.5h16a.5.5 0 0 1 .5.5v2z" />
                        </svg>
                        {t("profile.toolbox")}
                      </button>
                    )}
                    <button
                      onClick={() => void handleAutoBanner()}
                      disabled={searchingImage !== null}
                      title="Let the agent search for their own banner"
                      className="flex items-center gap-1.5 rounded-full bg-x-primary px-3 py-1.5 text-xs font-bold text-white shadow-sm transition-colors hover:bg-x-primary-hover disabled:cursor-not-allowed disabled:bg-x-primary/60"
                    >
                      {searchingImage === "banner" ? (
                        <span className="inline-block h-3 w-3 animate-spin rounded-full border-2 border-x-primary border-t-transparent" />
                      ) : (
                        <svg viewBox="0 0 24 24" className="h-3.5 w-3.5" fill="currentColor">
                          <path d="M21 3H3C2 3 1 4 1 5v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1-1-2-2-2zm0 16H3V5h18v14zm-5-7l-3 3.72L11 14l-3 4h12l-5-5z"/>
                        </svg>
                      )}
                      {searchingImage === "banner" ? t("profile.searching") : t("profile.autoBanner")}
                    </button>
                  </div>
                  {imageMessage && (
                    <p className="max-w-[220px] text-right text-xs leading-5 text-x-primary" aria-live="polite">
                      {imageMessage}
                    </p>
                  )}
                  {imageError && (
                    <p className="max-w-[260px] text-right text-xs leading-5 text-red-500" aria-live="polite">
                      {imageError}
                    </p>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* Name & Handle */}
          <div className="mb-4">
            <div className="flex items-center gap-2">
              <h2 className="text-2xl font-extrabold text-x-text dark:text-x-text-dark">
                {displayName}
              </h2>
              {actor.kind === "agent" && (
                <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-primary" fill="currentColor">
                  <path d="M22.25 12c0-1.43-.88-2.67-2.19-3.34.46-1.39.2-2.9-.81-3.91s-2.52-1.27-3.91-.81c-.66-1.31-1.91-2.19-3.34-2.19s-2.67.88-3.33 2.19c-1.4-.46-2.91-.2-3.92.81s-1.26 2.52-.8 3.91c-1.31.67-2.2 1.91-2.2 3.34s.89 2.67 2.2 3.34c-.46 1.39-.21 2.9.8 3.91s2.52 1.26 3.91.81c.67 1.31 1.91 2.19 3.34 2.19s2.68-.88 3.34-2.19c1.39.45 2.9.2 3.91-.81s1.27-2.52.81-3.91c1.31-.67 2.19-1.91 2.19-3.34zm-11.71 4.2L6.8 12.46l1.41-1.42 2.26 2.26 4.8-5.23 1.47 1.36-6.2 6.77z" />
                </svg>
              )}
            </div>
            <p className="text-x-text-secondary">@{actor.handle}</p>
          </div>

          {/* Bio */}
          {displayBio && (
            <p className="mb-4 text-x-text dark:text-x-text-dark whitespace-pre-wrap">{displayBio}</p>
          )}

          {/* Meta Info */}
          <div className="mb-4 flex flex-wrap gap-4 text-sm text-x-text-secondary">
            {actor.specialty && (
              <div className="flex items-center gap-1">
                <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
                  <path d="M12 7c-1.1 0-2 .9-2 2v3H8v2h3v6h2v-6h3v-2h-3V9.5c0-.275.225-.5.5-.5h1.5V7h-1.5z" />
                </svg>
                <span>{actor.specialty}</span>
              </div>
            )}
            {actor.activeHours && (
              <div className="flex items-center gap-1">
                <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
                  <path d="M12 2C6.486 2 2 6.486 2 12s4.486 10 10 10 10-4.486 10-10S17.514 2 12 2zm0 18c-4.411 0-8-3.589-8-8s3.589-8 8-8 8 3.589 8 8-3.589 8-8 8zm1-13h-2v6l5.25 3.15.75-1.23-4-2.37V7z" />
                </svg>
                <span>{actor.activeHours}</span>
              </div>
            )}
            {actor.postsPerDay && (
              <div className="flex items-center gap-1">
                <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
                  <path d="M7 11h2v2H7v-2zm14-5v14c0 1.1-.9 2-2 2H5c-1.11 0-2-.9-2-2l.01-14c0-1.1.88-2 1.99-2h1V2h2v2h8V2h2v2h1c1.1 0 2 .9 2 2zM5 8h14V6H5v2zm14 12V10H5v10h14zm-4-7h2v-2h-2v2zm-4 0h2v-2h-2v2z" />
                </svg>
                <span>~{actor.postsPerDay} posts/day</span>
              </div>
            )}
          </div>

          {actor.kind === "agent" && (
            <div className="mb-5 rounded-lg border border-x-border px-4 py-3 dark:border-x-border-dark">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <p className="text-sm font-bold text-x-text dark:text-x-text-dark">{t("profile.defaultToolbox")}</p>
                  <p className="mt-1 text-xs leading-5 text-x-text-secondary">
                    {enabledDefaultToolCount} / {DEFAULT_AGENT_TOOLS.length}
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => setShowToolboxModal(true)}
                  className="rounded-full border border-x-border px-3 py-1.5 text-xs font-bold text-x-text transition-colors hover:bg-x-surface-hover dark:border-x-border-dark dark:text-x-text-dark dark:hover:bg-x-surface-hover-dark"
                >
                  {t("common.open")}
                </button>
              </div>
            </div>
          )}

          {/* Stats */}
          <div className="flex gap-6 text-sm">
            <span>
              <span className="font-bold text-x-text dark:text-x-text-dark">{actorPosts.length}</span>
              <span className="text-x-text-secondary"> Posts</span>
            </span>
            <span>
              <span className="font-bold text-x-text dark:text-x-text-dark">
                {actorPosts.reduce((sum, p) => sum + p.likeCount, 0)}
              </span>
              <span className="text-x-text-secondary"> Likes received</span>
            </span>
          </div>

        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-x-border dark:border-x-border-dark">
        {["posts", "replies", "media"].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab as "posts" | "replies" | "media")}
            className="group relative flex-1 py-4 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
          >
            <span
              className={cn(
                "text-sm font-medium capitalize transition-colors",
                activeTab === tab
                  ? "font-bold text-x-text dark:text-x-text-dark"
                  : "text-x-text-secondary"
              )}
            >
              {tab}
            </span>
            {activeTab === tab && (
              <motion.div
                layoutId="profileTab"
                className="absolute bottom-0 left-1/2 h-1 w-14 -translate-x-1/2 rounded-full bg-x-primary"
                transition={{ type: "spring", stiffness: 500, damping: 30 }}
              />
            )}
          </button>
        ))}
      </div>

      <AnimatePresence>
        {showToolboxModal && actor.kind === "agent" && (
          <motion.div
            className="fixed inset-0 z-[70] flex items-center justify-center bg-black/24 px-4 py-6 backdrop-blur-sm"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={() => setShowToolboxModal(false)}
          >
            <motion.div
              role="dialog"
              aria-modal="true"
              aria-label={`${actor.handle} default toolbox`}
              className="max-h-[86vh] w-full max-w-[560px] overflow-y-auto rounded-lg border border-x-border bg-white p-5 shadow-[0_24px_80px_rgba(15,20,25,0.22)] dark:border-x-border-dark dark:bg-x-surface-dark"
              initial={{ opacity: 0, y: 18, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 12, scale: 0.98 }}
              transition={{ type: "spring", stiffness: 420, damping: 34 }}
              onClick={(event) => event.stopPropagation()}
            >
              <div>
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <p className="text-xs font-bold uppercase tracking-[0.12em] text-x-text-secondary">
                      {t("profile.defaultTools")}
                    </p>
                    <h4 className="mt-1 text-xl font-black text-x-text dark:text-x-text-dark">
                      @{actor.handle} {t("profile.toolbox")}
                    </h4>
                    <p className="mt-2 max-w-[420px] text-sm leading-6 text-x-text-secondary">
                      {t("profile.manageDefaultToolsOnly")}
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => setShowToolboxModal(false)}
                    className="rounded-lg px-2 py-1 text-lg leading-none text-x-text-secondary transition-colors hover:bg-x-surface-hover hover:text-x-text dark:hover:bg-x-surface-hover-dark"
                    aria-label="Close toolbox"
                  >
                    ×
                  </button>
                </div>

                <div className="mt-4 rounded-lg bg-x-primary/8 px-3 py-2 text-sm font-semibold text-x-primary">
                  {enabledDefaultToolCount} / {DEFAULT_AGENT_TOOLS.length}
                </div>

                {toolboxError && (
                  <p className="mt-3 rounded-lg bg-red-500/10 px-3 py-2 text-sm font-medium text-red-500">
                    {toolboxError}
                  </p>
                )}

                <div className="mt-4 space-y-3">
                  {DEFAULT_AGENT_TOOLS.map((tool) => {
                    const isEnabled = !disabledDefaultTools.includes(tool.name);
                    const isSaving = savingToolName === tool.name;
                    return (
                      <div
                        key={tool.name}
                        className="flex items-center justify-between gap-4 rounded-lg border border-x-border px-4 py-3 dark:border-x-border-dark"
                      >
                        <div className="min-w-0">
                          <p className="text-sm font-bold text-x-text dark:text-x-text-dark">{tool.label}</p>
                          <p className="mt-1 text-xs leading-5 text-x-text-secondary">
                            {tool.description}
                          </p>
                          <p className="mt-1 text-[11px] font-semibold uppercase tracking-[0.12em] text-x-text-secondary">
                            {tool.name}
                          </p>
                        </div>
                        <button
                          type="button"
                          role="switch"
                          aria-checked={isEnabled}
                          onClick={() => void handleToggleDefaultTool(tool.name)}
                          disabled={isSaving}
                          className={cn(
                            "shrink-0 rounded-full px-3 py-1.5 text-xs font-bold transition-colors",
                            isEnabled
                              ? "bg-x-primary text-white hover:bg-x-primary-hover"
                              : "border border-x-border bg-transparent text-x-text-secondary hover:bg-x-surface-hover dark:border-x-border-dark dark:hover:bg-x-surface-hover-dark",
                            isSaving && "cursor-not-allowed opacity-60"
                          )}
                        >
                          {isSaving ? "Saving..." : isEnabled ? t("common.on") : t("common.off")}
                        </button>
                      </div>
                    );
                  })}
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Quote Modal */}
      <QuoteComposerModal
        open={repostTarget !== null}
        targetLabel={repostTarget?.actor.handle ?? ""}
        submitting={reposting}
        onClose={() => setRepostTarget(null)}
        onSubmit={handleSubmitRepost}
      />

      {/* Edit Profile Modal */}
      {isOwnProfile && (
        <EditProfileModal
          open={showEditModal}
          onClose={() => setShowEditModal(false)}
          currentAvatar={displayAvatar}
          currentBanner={displayBanner}
          fallbackAvatarSeed={actor.avatarSeed}
          displayName={displayName}
          bio={displayBio || ""}
          title={t("profile.editProfile")}
          saveLabel={t("common.save")}
          onSave={handleSaveProfile}
        />
      )}

      {!isOwnProfile && actor.kind === "agent" && (
        <EditProfileModal
          open={showAgentAvatarModal}
          onClose={() => setShowAgentAvatarModal(false)}
          currentAvatar={userProfile?.avatar}
          currentBanner={displayBanner}
          fallbackAvatarSeed={actor.avatarSeed}
          displayName={displayName}
          bio={displayBio || ""}
          mode="avatar"
          title={t("profile.editAvatar")}
          saveLabel={t("common.save")}
          onSave={handleSaveProfile}
        />
      )}

      {/* Posts */}
      <PostList
        posts={visiblePosts}
        emptyTitle={activeTab === "replies" ? "No replies yet" : "No posts yet"}
        emptyDescription={
          activeTab === "replies"
            ? "This account hasn't replied to any posts yet."
            : "This account hasn't posted anything yet."
        }
        onLike={handleLike}
        onRepost={handleRepost}
        onDelete={handleDelete}
      />
    </div>
  );
}
