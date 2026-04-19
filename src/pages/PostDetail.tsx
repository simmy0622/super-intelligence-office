import { useCallback, useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { ThreadView } from "../components/ThreadView";
import { QuoteComposerModal } from "../components/QuoteComposerModal";
import { updatePost, upsertPost } from "../lib/post-state";
import {
  deletePost,
  getProfile,
  getThread,
  likeToggle,
  replyAsHuman,
  repostAsHuman,
  type FeedPost,
  type UserProfile,
} from "../lib/client";

export function PostDetail() {
  const { id = "" } = useParams();
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  const postId = Number.parseInt(id, 10);
  const focusReply = searchParams.get("reply") === "1";

  const [thread, setThread] = useState<FeedPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [repostTarget, setRepostTarget] = useState<FeedPost | null>(null);
  const [reposting, setReposting] = useState(false);
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);

  const refreshThread = useCallback(async () => {
    if (!Number.isFinite(postId)) {
      setError("Invalid post id.");
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      setThread(await getThread(postId));
      setError(null);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to load post.");
    } finally {
      setLoading(false);
    }
  }, [postId]);

  useEffect(() => {
    void refreshThread();
    void getProfile("You").then(setUserProfile);
  }, [refreshThread]);

  const handleLike = useCallback((targetId: number) => {
    setThread((current) =>
      updatePost(current, targetId, (post) => ({
        ...post,
        likedByYou: !post.likedByYou,
        likeCount: Math.max(0, post.likeCount + (post.likedByYou ? -1 : 1)),
      }))
    );
    likeToggle(targetId).catch(() => {
      setThread((current) =>
        updatePost(current, targetId, (post) => ({
          ...post,
          likedByYou: !post.likedByYou,
          likeCount: Math.max(0, post.likeCount + (post.likedByYou ? -1 : 1)),
        }))
      );
    });
  }, []);

  const handleRepost = async (targetId: number) => {
    const target = thread.find((post) => post.id === targetId) ?? null;
    setRepostTarget(target);
  };

  const handleDelete = async (targetId: number) => {
    const result = await deletePost(targetId);
    const deleted = new Set(result.deletedPostIds);
    if (deleted.has(postId)) {
      navigate("/");
      return;
    }
    setThread((current) => current.filter((post) => !deleted.has(post.id)));
  };

  const handleSubmitRepost = async (quoteBody: string | null) => {
    if (!repostTarget) return;
    setReposting(true);
    try {
      const reposted = await repostAsHuman(repostTarget.id, quoteBody, repostTarget.salonId);
      setThread((current) => {
        const withCount = updatePost(current, repostTarget.id, (post) => ({
          ...post,
          repostCount: post.repostCount + 1,
        }));
        return upsertPost(withCount, reposted);
      });
      setRepostTarget(null);
    } finally {
      setReposting(false);
    }
  };

  const handleReply = async (body: string) => {
    const target = thread.find((post) => post.id === postId) ?? thread[0];
    await replyAsHuman(postId, body, target?.salonId ?? null);
    await refreshThread();
  };

  const rootPost = thread[0];
  const replyTargetHandle = rootPost?.actor.handle ?? null;

  return (
    <div>
      <ThreadView
        posts={thread}
        loading={loading}
        error={error}
        onClose={() => navigate(-1)}
        onRetry={refreshThread}
        onReply={handleReply}
        onLike={handleLike}
        onRepost={handleRepost}
        onDelete={handleDelete}
        replyTargetHandle={replyTargetHandle}
        avatar={userProfile?.avatar}
        userHandle={userProfile?.displayName || "You"}
        autoFocusReply={focusReply}
      />
      <QuoteComposerModal
        open={repostTarget !== null}
        targetLabel={repostTarget?.actor.handle ?? ""}
        submitting={reposting}
        onClose={() => setRepostTarget(null)}
        onSubmit={handleSubmitRepost}
      />
    </div>
  );
}
