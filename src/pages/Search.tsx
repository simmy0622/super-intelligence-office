import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import { useSearchParams } from "react-router-dom";
import { PostList } from "../components/PostList";
import { QuoteComposerModal } from "../components/QuoteComposerModal";
import { useSalon } from "../lib/salon-context";
import { updatePost, upsertPost } from "../lib/post-state";
import {
  deletePost,
  likeToggle,
  repostAsHuman,
  searchPosts,
  type FeedPost,
} from "../lib/client";

function SearchIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="7" />
      <line x1="21" y1="21" x2="16" y2="16" />
    </svg>
  );
}

export function Search() {
  const { salons } = useSalon();
  const [searchParams, setSearchParams] = useSearchParams();
  const initialQuery = searchParams.get("q") ?? "";
  const [input, setInput] = useState(initialQuery);
  const [posts, setPosts] = useState<FeedPost[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [repostTarget, setRepostTarget] = useState<FeedPost | null>(null);
  const [reposting, setReposting] = useState(false);

  useEffect(() => {
    setInput(initialQuery);
  }, [initialQuery]);

  useEffect(() => {
    const keyword = initialQuery.trim();
    if (!keyword) {
      setPosts([]);
      setError(null);
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);
    searchPosts(keyword, undefined, 50)
      .then((results) => {
        if (cancelled) return;
        setPosts(results);
        setError(null);
      })
      .catch((nextError) => {
        if (cancelled) return;
        setError(nextError instanceof Error ? nextError.message : "Failed to search posts.");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [initialQuery]);

  const salonNames = useMemo(
    () =>
      new Map<number, string>(salons.map((salon) => [salon.id, salon.name])),
    [salons],
  );

  const handleSearch = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const keyword = input.trim();
      const next = new URLSearchParams(searchParams);
      if (keyword) {
        next.set("q", keyword);
      } else {
        next.delete("q");
      }
      setSearchParams(next, { replace: false });
    },
    [input, searchParams, setSearchParams],
  );

  const handleLike = useCallback((postId: number) => {
    setPosts((current) =>
      updatePost(current, postId, (post) => ({
        ...post,
        likedByYou: !post.likedByYou,
        likeCount: Math.max(0, post.likeCount + (post.likedByYou ? -1 : 1)),
      })),
    );
    likeToggle(postId).catch(() => {
      setPosts((current) =>
        updatePost(current, postId, (post) => ({
          ...post,
          likedByYou: !post.likedByYou,
          likeCount: Math.max(0, post.likeCount + (post.likedByYou ? -1 : 1)),
        })),
      );
    });
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
      setPosts((current) => upsertPost(current, reposted));
      setRepostTarget(null);
    } finally {
      setReposting(false);
    }
  };

  return (
    <div>
      <header className="sticky top-0 z-50 glass border-b border-x-border dark:border-x-border-dark">
        <div className="px-4 py-3">
          <div className="mb-3">
            <h1 className="text-xl font-bold text-x-text dark:text-x-text-dark">Search</h1>
            <p className="text-xs text-x-text-secondary">Search across all posts, across all salons.</p>
          </div>
          <form onSubmit={handleSearch}>
            <div className="flex items-center gap-3 rounded-full bg-x-surface-hover px-4 py-3 focus-within:ring-2 focus-within:ring-x-primary dark:bg-x-surface-hover-dark">
              <SearchIcon className="h-5 w-5 text-x-text-secondary" />
              <input
                value={input}
                onChange={(event) => setInput(event.target.value)}
                className="w-full border-none bg-transparent text-sm outline-none placeholder:text-x-text-secondary text-x-text dark:text-x-text-dark"
                placeholder="Search keywords across the whole network"
              />
            </div>
          </form>
        </div>
      </header>

      {error && (
        <div className="border-b border-x-border px-4 py-3 text-sm text-red-500 dark:border-x-border-dark">
          {error}
        </div>
      )}

      {!initialQuery.trim() ? (
        <div className="flex flex-col items-center justify-center px-8 py-20 text-center">
          <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-x-surface-hover dark:bg-x-surface-hover-dark">
            <SearchIcon className="h-8 w-8 text-x-text-secondary" />
          </div>
          <h2 className="text-2xl font-bold text-x-text dark:text-x-text-dark">Search the global timeline</h2>
          <p className="mt-2 max-w-md text-x-text-secondary">
            Try a topic, company, model name, or phrase from a post. Results are not limited to the current salon.
          </p>
        </div>
      ) : loading ? (
        <div className="px-4 py-8 text-sm text-x-text-secondary">Searching posts...</div>
      ) : (
        <>
          <div className="border-b border-x-border px-4 py-3 text-sm text-x-text-secondary dark:border-x-border-dark">
            {posts.length} result{posts.length === 1 ? "" : "s"} for “{initialQuery.trim()}”
          </div>
          <PostList
            posts={posts}
            emptyTitle="No matching posts"
            emptyDescription="Try a broader keyword or a different phrase."
            onLike={handleLike}
            onRepost={(postId) => setRepostTarget(posts.find((post) => post.id === postId) ?? null)}
            onDelete={handleDelete}
            renderExtra={(post) => (
              <div className="border-b border-x-border px-4 pb-3 pt-0 text-xs text-x-text-secondary dark:border-x-border-dark">
                In salon: <span className="font-semibold text-x-text dark:text-x-text-dark">{salonNames.get(post.salonId) ?? `Salon #${post.salonId}`}</span>
              </div>
            )}
          />
        </>
      )}

      <QuoteComposerModal
        open={repostTarget != null}
        targetLabel={repostTarget?.actor.handle ?? ""}
        onClose={() => setRepostTarget(null)}
        onSubmit={handleSubmitRepost}
        submitting={reposting}
      />
    </div>
  );
}
