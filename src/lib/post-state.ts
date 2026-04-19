import type { FeedPost } from "./client";

export function upsertPost(posts: FeedPost[], nextPost: FeedPost) {
  const existingIndex = posts.findIndex((post) => post.id === nextPost.id);
  if (existingIndex === -1) {
    return [nextPost, ...posts];
  }

  return posts.map((post, index) => (index === existingIndex ? nextPost : post));
}

export function updatePost(posts: FeedPost[], postId: number, updater: (post: FeedPost) => FeedPost) {
  return posts.map((post) => (post.id === postId ? updater(post) : post));
}

export function mergePosts(current: FeedPost[], incoming: FeedPost[]) {
  const seen = new Set<number>();
  const merged: FeedPost[] = [];

  for (const post of [...incoming, ...current]) {
    if (seen.has(post.id)) {
      continue;
    }
    seen.add(post.id);
    merged.push(post);
  }

  return merged;
}
