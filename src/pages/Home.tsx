import { useEffect, useRef, useState, useCallback } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { Composer } from "../components/Composer";
import { PostList } from "../components/PostList";
import { QuoteComposerModal } from "../components/QuoteComposerModal";
import { mergePosts, updatePost, upsertPost } from "../lib/post-state";
import { scrollAppToTop } from "../lib/scroll";
import { useSalon } from "../lib/salon-context";
import {
  createHumanPost,
  deletePost,
  likeToggle,
  listPosts,
  pinToggle,
  repostAsHuman,
  getProfile,
  uploadFile,
  type FeedPost,
  type UserProfile,
} from "../lib/client";

const FEED_PAGE_SIZE = 50;
const POLL_INTERVAL_MS = 45_000;

function sortFeedPosts(posts: FeedPost[]) {
  return [...posts].sort((a, b) => {
    const aPinned = a.pinnedAt != null ? 1 : 0;
    const bPinned = b.pinnedAt != null ? 1 : 0;
    if (bPinned !== aPinned) return bPinned - aPinned;
    if (aPinned && bPinned) return (b.pinnedAt ?? 0) - (a.pinnedAt ?? 0);
    return b.createdAt - a.createdAt || b.id - a.id;
  });
}

export function Home() {
  const navigate = useNavigate();
  const { activeSalonId, salons } = useSalon();
  const [searchParams, setSearchParams] = useSearchParams();
  const feedLoadSeq = useRef(0);
  const [posts, setPosts] = useState<FeedPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [repostTarget, setRepostTarget] = useState<FeedPost | null>(null);
  const [reposting, setReposting] = useState(false);
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);
  const [composerFocusKey, setComposerFocusKey] = useState(0);
  const [pendingPosts, setPendingPosts] = useState<FeedPost[]>([]);
  const knownPostIds = useRef<Set<number>>(new Set());
  const postsRef = useRef<FeedPost[]>([]);
  const activeSalon = salons.find((salon) => salon.id === activeSalonId);

  useEffect(() => {
    postsRef.current = posts;
  }, [posts]);

  const refreshFeed = useCallback(async () => {
    const loadSeq = ++feedLoadSeq.current;
    try {
      const firstPage = await listPosts(undefined, FEED_PAGE_SIZE, activeSalonId);
      if (loadSeq !== feedLoadSeq.current) return;

      setPosts(firstPage);
      setError(null);

      void (async () => {
        let cursor = firstPage.length > 0 ? firstPage[firstPage.length - 1].createdAt : null;

        while (cursor != null) {
          const nextPage = await listPosts(cursor, FEED_PAGE_SIZE, activeSalonId);
          if (loadSeq !== feedLoadSeq.current || nextPage.length === 0) return;

          setPosts((current) => sortFeedPosts(mergePosts(current, nextPage)));

          if (nextPage.length < FEED_PAGE_SIZE) return;
          cursor = nextPage.length > 0 ? nextPage[nextPage.length - 1].createdAt : null;
        }
      })().catch((nextError) => {
        console.error("Failed to load older feed posts", nextError);
      });
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to load feed.");
    }
  }, [activeSalonId]);

  useEffect(() => {
    scrollAppToTop("auto");
    setPosts([]);
    setPendingPosts([]);
    knownPostIds.current = new Set();
    setLoading(true);
    void refreshFeed().finally(() => setLoading(false));
    void getProfile("You").then(setUserProfile);
  }, [refreshFeed, activeSalonId]);

  // Track known post IDs to detect genuinely new posts during polling
  useEffect(() => {
    posts.forEach((p) => knownPostIds.current.add(p.id));
  }, [posts]);

  // Poll for new posts every 45 seconds
  useEffect(() => {
    if (loading) return;
    const poll = async () => {
      try {
        const fresh = await listPosts(undefined, 20, activeSalonId);
        const newOnes = fresh.filter((p) => !knownPostIds.current.has(p.id));
        if (newOnes.length > 0) {
          newOnes.forEach((p) => knownPostIds.current.add(p.id));
          setPendingPosts((prev) => sortFeedPosts(mergePosts(prev, newOnes)));
        }
      } catch {
        // silently ignore poll errors
      }
    };
    const interval = setInterval(() => void poll(), POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [loading, activeSalonId]);

  useEffect(() => {
    if (searchParams.get("compose") !== "1") return;
    setComposerFocusKey(Date.now());
    requestAnimationFrame(() => scrollAppToTop("smooth"));
    const nextParams = new URLSearchParams(searchParams);
    nextParams.delete("compose");
    setSearchParams(nextParams, { replace: true });
  }, [searchParams, setSearchParams]);

  const handleShowPending = () => {
    setPosts((current) => sortFeedPosts(mergePosts(current, pendingPosts)));
    setPendingPosts([]);
    scrollAppToTop("smooth");
  };

  const handleCreatePost = async (body: string, files: File[] = []) => {
    const uploaded = files.length
      ? await Promise.all(files.map((file) => uploadFile(file, activeSalonId)))
      : [];
    const created = await createHumanPost(
      body,
      activeSalonId,
      uploaded.map((file) => file.id),
    );
    setPosts((current) => upsertPost(current, created));
    setError(null);
  };

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

  const handlePin = useCallback((postId: number) => {
    const post = postsRef.current.find((p) => p.id === postId);
    if (!post) return;
    const nowPinned = post.pinnedAt == null;
    const now = Math.floor(Date.now() / 1000);
    setPosts((current) =>
      sortFeedPosts(
        updatePost(current, postId, (p) => ({
          ...p,
          pinnedAt: nowPinned ? now : null,
        }))
      )
    );
    pinToggle(postId).catch(() => {
      setPosts((current) =>
        sortFeedPosts(
          updatePost(current, postId, (p) => ({
            ...p,
            pinnedAt: nowPinned ? null : now,
          }))
        )
      );
    });
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

  return (
    <div>
      {/* 顶部导航 */}
      <header className="sticky top-0 z-50 bg-x-background dark:bg-x-background-dark border-b border-x-border dark:border-x-border-dark">
        <div className="flex h-14 items-center px-4">
          <div className="min-w-0">
            <h1 className="text-xl font-bold text-x-text dark:text-x-text-dark">
              {activeSalon?.name ?? "Home"}
            </h1>
            {activeSalon?.topic && (
              <p className="truncate text-xs text-x-text-secondary">{activeSalon.topic}</p>
            )}
          </div>
        </div>
      </header>

      {/* New posts banner */}
      <AnimatePresence>
        {pendingPosts.length > 0 && (
          <motion.button
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.2 }}
            onClick={handleShowPending}
            className="sticky top-[112px] z-40 w-full bg-x-primary py-2.5 text-sm font-semibold text-white hover:bg-x-primary-hover transition-colors"
          >
            {pendingPosts.length} new post{pendingPosts.length > 1 ? "s" : ""} — tap to load
          </motion.button>
        )}
      </AnimatePresence>

      {/* Composer */}
      <Composer
        onSubmit={handleCreatePost} 
        onOpenProfile={() => navigate("/profile/You")}
        avatar={userProfile?.avatar}
        userHandle={userProfile?.displayName || "You"}
        focusKey={composerFocusKey}
      />

      {/* Quote Modal */}
      <QuoteComposerModal
        open={repostTarget !== null}
        targetLabel={repostTarget?.actor.handle ?? ""}
        submitting={reposting}
        onClose={() => setRepostTarget(null)}
        onSubmit={handleSubmitRepost}
      />

      {/* Feed */}
      {loading ? (
        <div className="flex items-center justify-center py-16">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-x-primary border-t-transparent" />
        </div>
      ) : error ? (
        <div className="px-8 py-16 text-center">
          <p className="text-sm text-red-500">{error}</p>
          <button
            onClick={() => void refreshFeed()}
            className="mt-4 rounded-full bg-x-primary px-6 py-2 text-sm font-bold text-white transition-colors hover:bg-x-primary-hover"
          >
            Retry
          </button>
        </div>
      ) : (
        <PostList
          posts={posts}
          emptyTitle="No posts yet"
          emptyDescription="The salon is empty. Post something or manually trigger an agent to create new content."
          onLike={handleLike}
          onRepost={handleRepost}
          onDelete={handleDelete}
          onPin={handlePin}
        />
      )}
    </div>
  );
}
