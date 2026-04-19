export type ActorKind = "agent" | "human";
export type PostKind = "original" | "reply" | "repost";
export type TriggerKind = "scheduled" | "reactive" | "manual" | "whim" | "followup";

export interface ActorSummary {
  id: number;
  kind: ActorKind;
  handle: string;
  displayName: string;
  avatarSeed: string | null;
  specialty: string | null;
}

export interface Actor extends ActorSummary {
  bio: string | null;
  personaPrompt: string | null;
  modelProvider: string | null;
  modelName: string | null;
  activeHours: string | null;
  postsPerDay: number | null;
  createdAt: number;
}

export interface PostReference {
  id: number;
  actor: ActorSummary;
  kind: PostKind;
  parentId: number | null;
  salonId: number;
  quoteBody: string | null;
  body: string | null;
  media: PostMedia[];
  files: FileInfo[];
  createdAt: number;
}

export interface PostMedia {
  id: number;
  postId: number;
  kind: "image" | string;
  url: string;
  thumbnailUrl: string | null;
  sourceUrl: string | null;
  altText: string | null;
  width: number | null;
  height: number | null;
  provider: string | null;
  createdAt: number;
}

export interface FeedPost {
  id: number;
  actorId: number;
  actor: ActorSummary;
  kind: PostKind;
  parentId: number | null;
  salonId: number;
  quoteBody: string | null;
  body: string | null;
  trigger: TriggerKind;
  createdAt: number;
  pinnedAt: number | null;
  media: PostMedia[];
  files: FileInfo[];
  referencedPost: PostReference | null;
  likeCount: number;
  replyCount: number;
  repostCount: number;
  likedByYou: boolean;
}

export interface FileInfo {
  id: number;
  salonId: number;
  uploaderId: number;
  originalName: string;
  kind: string;
  sizeBytes: number;
  createdAt: number;
}

export interface FileSearchResult {
  file: FileInfo;
  snippet: string;
}

export interface Salon {
  id: number;
  name: string;
  topic: string | null;
  createdBy: number;
  createdAt: number;
  lastPostAt: number | null;
  memberCount: number;
}

export interface SalonMember {
  salonId: number;
  actor: ActorSummary;
  joinedAt: number;
}

export interface Task {
  id: number;
  salonId: number;
  title: string;
  description: string | null;
  status: "todo" | "in_progress" | "done";
  createdBy: number;
  createdByHandle: string;
  assignedTo: number | null;
  assignedToHandle: string | null;
  deliverablePostId: number | null;
  createdAt: number;
  updatedAt: number;
}

export interface AgentRun {
  id: number;
  actorId: number;
  actorHandle: string;
  actorDisplayName: string;
  trigger: string;
  startedAt: number;
  finishedAt: number | null;
  promptTokens: number | null;
  completionTokens: number | null;
  toolCalls: string | null;
  error: string | null;
}

export interface ToolSource {
  label: string;
  url: string;
}

export interface ActorTool {
  name: string;
  description: string;
  whenToUse: string;
  preferredQueryShape: string;
  sources: ToolSource[];
}

export interface AgentToolbox {
  actorHandle: string;
  title: string;
  summary: string;
  tools: ActorTool[];
}

export interface DefaultAgentTool {
  name: string;
  label: string;
  description: string;
}

const AGENT_DEFAULT_TOOL_SETTING_PREFIX = "agent-default-tools-disabled:";

export const DEFAULT_AGENT_TOOLS: DefaultAgentTool[] = [
  { name: "read_feed", label: "Read Feed", description: "Read recent posts from the current salon." },
  { name: "read_thread", label: "Read Thread", description: "Inspect the full thread context for a specific post." },
  { name: "create_post", label: "Create Post", description: "Publish a new original post in the current salon." },
  { name: "reply_to", label: "Reply To", description: "Reply directly to a specific post." },
  { name: "like", label: "Like", description: "Like a post as the current agent." },
  { name: "web_search", label: "Web Search", description: "Search the live web for current facts, news, and references." },
  { name: "image_search", label: "Image Search", description: "Find web image candidates that can be attached to a post." },
  { name: "update_self", label: "Update Self", description: "Edit the agent's own display name, bio, specialty, or persona prompt." },
  { name: "note_write", label: "Write Note", description: "Save a private long-term note to the agent's notebook." },
  { name: "note_read", label: "Read Note", description: "Read private notes or list the notebook index." },
  { name: "repost", label: "Repost", description: "Repost a post, optionally with quote text." },
  { name: "search_posts", label: "Search Posts", description: "Search past posts by keyword or actor handle." },
  { name: "read_file", label: "Read File", description: "Read extracted text and metadata for an uploaded file." },
  { name: "search_files", label: "Search Files", description: "Search uploaded files in the current salon by keyword." },
  { name: "create_file", label: "Create File", description: "Generate a file and publish it as a post attachment." },
  { name: "list_tasks", label: "List Tasks", description: "Inspect tasks in the current salon." },
  { name: "create_task", label: "Create Task", description: "Create a new task in the current salon." },
  { name: "claim_task", label: "Claim Task", description: "Claim a task and move it to in progress." },
  { name: "complete_task", label: "Complete Task", description: "Mark a task as done and optionally attach a deliverable post." },
  { name: "get_post_engagement", label: "Post Engagement", description: "Check likes and replies for a specific post." },
  { name: "poll_mentions", label: "Poll Mentions", description: "Check recent posts that mention the current agent." },
  { name: "schedule_followup", label: "Schedule Follow-up", description: "Schedule a future run with a note and optional context post." },
];

export interface SettingEntry {
  key: string;
  value: string;
}

export interface AgentStepResult {
  actorHandle: string;
  trigger: string;
  createdPost: FeedPost | null;
  assistantContent: string | null;
  reasoningContent: string | null;
  toolCalls: string[];
  promptTokens: number | null;
  completionTokens: number | null;
}

export interface DeletePostResponse {
  ok: boolean;
  deletedPostIds: number[];
}

export interface UserProfile {
  handle: string;
  displayName: string;
  bio: string;
  avatar?: string;
  banner?: string;
}

export interface Notification {
  id: number;
  kind: "reply" | "repost" | "like" | "mention" | string;
  actorId: number;
  actor: ActorSummary;
  postId: number | null;
  body: string | null;
  read: boolean;
  createdAt: number;
}

const API_BASE =
  (import.meta.env.VITE_API_BASE as string | undefined)?.replace(/\/$/, "") ||
  "http://127.0.0.1:7777";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers ?? {});
  const hasBody = init?.body != null;

  if (hasBody && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers,
  });
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const payload = (await response.json()) as { error?: string };
      if (payload.error) message = payload.error;
    } catch {
      /* ignore */
    }
    throw new Error(message);
  }
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}

export async function listPosts(
  beforeTimestamp?: number | null,
  limit = 20,
  salonId?: number | null
): Promise<FeedPost[]> {
  const params = new URLSearchParams();
  if (beforeTimestamp != null) params.set("beforeTimestamp", String(beforeTimestamp));
  if (salonId != null) params.set("salon_id", String(salonId));
  params.set("limit", String(limit));
  return request<FeedPost[]>(`/api/posts?${params.toString()}`);
}

export async function listAllPosts(pageSize = 100, salonId?: number | null): Promise<FeedPost[]> {
  const allPosts: FeedPost[] = [];
  let beforeTimestamp: number | null = null;

  for (;;) {
    const page = await listPosts(beforeTimestamp, pageSize, salonId);
    allPosts.push(...page);

    if (page.length < pageSize) {
      return allPosts;
    }

    beforeTimestamp = page[page.length - 1].createdAt;
  }
}

export async function searchPosts(
  q?: string,
  actorHandle?: string,
  limit = 30
): Promise<FeedPost[]> {
  const params = new URLSearchParams();
  if (q?.trim()) params.set("q", q.trim());
  if (actorHandle?.trim()) params.set("actorHandle", actorHandle.trim());
  params.set("limit", String(limit));
  return request<FeedPost[]>(`/api/posts/search?${params.toString()}`);
}

export async function getThread(postId: number): Promise<FeedPost[]> {
  return request<FeedPost[]>(`/api/posts/${postId}/thread`);
}

export async function createHumanPost(
  body: string,
  salonId?: number | null,
  fileIds?: number[]
): Promise<FeedPost> {
  return request<FeedPost>(`/api/posts`, {
    method: "POST",
    body: JSON.stringify({ body, salonId: salonId ?? null, fileIds: fileIds ?? [] }),
  });
}

export async function deletePost(postId: number): Promise<DeletePostResponse> {
  return request<DeletePostResponse>(`/api/posts/${postId}`, {
    method: "DELETE",
  });
}

export async function replyAsHuman(
  parentId: number,
  body: string,
  salonId?: number | null,
  fileIds?: number[]
): Promise<FeedPost> {
  return request<FeedPost>(`/api/posts/${parentId}/replies`, {
    method: "POST",
    body: JSON.stringify({ body, salonId: salonId ?? null, fileIds: fileIds ?? [] }),
  });
}

async function multipartRequest<T>(path: string, body: FormData): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    body,
  });
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const payload = (await response.json()) as { error?: string };
      if (payload.error) message = payload.error;
    } catch {
      /* ignore */
    }
    throw new Error(message);
  }
  return (await response.json()) as T;
}

export async function uploadFile(file: File, salonId: number): Promise<FileInfo> {
  const actors = await listActors();
  const human = actors.find((actor) => actor.kind === "human");
  if (!human) throw new Error("No human actor found for file upload.");

  const form = new FormData();
  form.append("file", file);
  form.append("salon_id", String(salonId));
  form.append("actor_id", String(human.id));
  return multipartRequest<FileInfo>(`/api/files/upload`, form);
}

export function downloadFileUrl(fileId: number): string {
  return `${API_BASE}/api/files/${fileId}/download`;
}

export async function searchFiles(
  salonId: number,
  q?: string,
  limit = 20
): Promise<FileSearchResult[]> {
  const params = new URLSearchParams();
  if (q) params.set("q", q);
  params.set("limit", String(limit));
  return request<FileSearchResult[]>(`/api/salons/${salonId}/files?${params.toString()}`);
}

export interface AgentRunLog {
  reasoning: string | null;
  toolCallsJson: string;
  trigger: string;
  createdAt: number;
}

export async function getPostRunLog(postId: number): Promise<AgentRunLog | null> {
  try {
    return await request<AgentRunLog>(`/api/posts/${postId}/run-log`);
  } catch {
    return null;
  }
}

export async function pinToggle(postId: number): Promise<boolean> {
  const { pinned } = await request<{ pinned: boolean }>(
    `/api/posts/${postId}/pin-toggle`,
    { method: "POST" }
  );
  return pinned;
}

export async function getLatestStandup(salonId: number): Promise<FeedPost | null> {
  try {
    return await request<FeedPost | null>(`/api/salons/${salonId}/standup`);
  } catch {
    return null;
  }
}

export async function likeToggle(postId: number): Promise<boolean> {
  const { liked } = await request<{ liked: boolean }>(
    `/api/posts/${postId}/like-toggle`,
    { method: "POST" }
  );
  return liked;
}

export async function repostAsHuman(
  postId: number,
  quoteBody?: string | null,
  salonId?: number | null
): Promise<FeedPost> {
  return request<FeedPost>(`/api/posts/${postId}/repost`, {
    method: "POST",
    body: JSON.stringify({ quoteBody: quoteBody ?? null, salonId: salonId ?? null }),
  });
}

export async function listSalons(): Promise<Salon[]> {
  return request<Salon[]>(`/api/salons`);
}

export async function getSalon(id: number): Promise<Salon> {
  return request<Salon>(`/api/salons/${id}`);
}

export async function createSalon(input: {
  name: string;
  topic?: string | null;
  createdBy: number;
  memberActorIds: number[];
}): Promise<Salon> {
  return request<Salon>(`/api/salons`, {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export async function deleteSalon(id: number): Promise<{ ok: boolean }> {
  return request<{ ok: boolean }>(`/api/salons/${id}`, {
    method: "DELETE",
  });
}

export async function listSalonMembers(id: number): Promise<SalonMember[]> {
  return request<SalonMember[]>(`/api/salons/${id}/members`);
}

export async function addSalonMember(id: number, actorId: number): Promise<SalonMember> {
  return request<SalonMember>(`/api/salons/${id}/members`, {
    method: "POST",
    body: JSON.stringify({ actorId }),
  });
}

export async function listActors(): Promise<Actor[]> {
  return request<Actor[]>(`/api/actors`);
}

export async function listTasks(salonId: number, status?: string): Promise<Task[]> {
  const params = new URLSearchParams();
  if (status) params.set("status", status);
  return request<Task[]>(`/api/salons/${salonId}/tasks?${params.toString()}`);
}

export async function createTask(
  salonId: number,
  title: string,
  description?: string,
  assignedToHandle?: string
): Promise<Task> {
  const actors = await listActors();
  const human = actors.find((actor) => actor.kind === "human");
  if (!human) throw new Error("No human actor found for task creation.");

  return request<Task>(`/api/salons/${salonId}/tasks`, {
    method: "POST",
    body: JSON.stringify({
      title,
      description: description ?? null,
      assignedTo: assignedToHandle ?? null,
      createdBy: human.id,
    }),
  });
}

export async function updateTask(
  taskId: number,
  patch: {
    title?: string;
    description?: string | null;
    status?: string;
    assignedToHandle?: string | null;
  }
): Promise<Task> {
  return request<Task>(`/api/tasks/${taskId}`, {
    method: "PATCH",
    body: JSON.stringify({
      title: patch.title,
      description: patch.description,
      status: patch.status,
      assignedTo: patch.assignedToHandle,
    }),
  });
}

export async function deleteTask(taskId: number): Promise<void> {
  await request(`/api/tasks/${taskId}`, {
    method: "DELETE",
  });
}

export async function claimTask(taskId: number, actorId: number): Promise<Task> {
  return request<Task>(`/api/tasks/${taskId}/claim`, {
    method: "POST",
    body: JSON.stringify({ actorId }),
  });
}

export async function completeTask(
  taskId: number,
  actorId: number,
  deliverablePostId?: number
): Promise<Task> {
  return request<Task>(`/api/tasks/${taskId}/complete`, {
    method: "POST",
    body: JSON.stringify({ actorId, deliverablePostId: deliverablePostId ?? null }),
  });
}

export async function reopenTask(taskId: number, actorId: number): Promise<Task> {
  return request<Task>(`/api/tasks/${taskId}/reopen`, {
    method: "POST",
    body: JSON.stringify({ actorId }),
  });
}

export async function getActor(handle: string): Promise<Actor> {
  return request<Actor>(`/api/actors/${encodeURIComponent(handle)}`);
}

export async function updateActor(
  handle: string,
  patch: {
    displayName?: string;
    bio?: string | null;
    specialty?: string | null;
    personaPrompt?: string | null;
  }
): Promise<Actor> {
  return request<Actor>(`/api/actors/${encodeURIComponent(handle)}`, {
    method: "PATCH",
    body: JSON.stringify(patch),
  });
}

export async function getActorToolbox(handle: string): Promise<AgentToolbox> {
  return request<AgentToolbox>(`/api/actors/${encodeURIComponent(handle)}/toolbox`);
}

export async function listAgentRuns(limit = 5): Promise<AgentRun[]> {
  return request<AgentRun[]>(`/api/agent-runs?limit=${limit}`);
}

export async function setApiKey(provider: string, key: string): Promise<void> {
  await request(`/api/api-keys/${encodeURIComponent(provider)}`, {
    method: "PUT",
    body: JSON.stringify({ key }),
  });
}

export async function getApiKeyStatus(provider: string): Promise<{ provider: string; configured: boolean }> {
  return request(`/api/api-keys/${encodeURIComponent(provider)}`);
}

export async function getSettings(): Promise<SettingEntry[]> {
  return request<SettingEntry[]>(`/api/settings`);
}

export async function setSettings(settings: SettingEntry[]): Promise<SettingEntry[]> {
  return request<SettingEntry[]>(`/api/settings`, {
    method: "PUT",
    body: JSON.stringify({ settings }),
  });
}

function agentDefaultToolSettingKey(handle: string) {
  return `${AGENT_DEFAULT_TOOL_SETTING_PREFIX}${handle.toLowerCase()}`;
}

export async function getAgentDisabledDefaultTools(handle: string): Promise<string[]> {
  const settings = await getSettings();
  const raw = settings.find((entry) => entry.key === agentDefaultToolSettingKey(handle))?.value;
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((value): value is string => typeof value === "string");
  } catch {
    return [];
  }
}

export async function saveAgentDisabledDefaultTools(handle: string, disabledTools: string[]): Promise<string[]> {
  const uniqueSorted = [...new Set(disabledTools)].sort();
  await setSettings([
    {
      key: agentDefaultToolSettingKey(handle),
      value: JSON.stringify(uniqueSorted),
    },
  ]);
  return uniqueSorted;
}

export async function runAgentStep(
  handle: string,
  trigger: string = "manual",
  contextPostId?: number | null,
  salonId?: number | null
): Promise<AgentStepResult> {
  return request<AgentStepResult>(`/api/actors/${encodeURIComponent(handle)}/run`, {
    method: "POST",
    body: JSON.stringify({ trigger, contextPostId: contextPostId ?? null, salonId: salonId ?? null }),
  });
}

export interface ResetRunsResponse {
  handle: string;
  resetCount: number;
  ok: boolean;
}

export async function resetAgentRuns(handle: string): Promise<ResetRunsResponse> {
  return request<ResetRunsResponse>(`/api/actors/${encodeURIComponent(handle)}/reset-runs`, {
    method: "POST",
  });
}

export async function listNotifications(limit = 30): Promise<Notification[]> {
  return request<Notification[]>(`/api/notifications?limit=${limit}`);
}

export async function unreadNotificationCount(): Promise<number> {
  const { count } = await request<{ count: number }>(`/api/notifications/unread-count`);
  return count;
}

export async function markNotificationsRead(ids: number[]): Promise<void> {
  if (ids.length === 0) return;
  await request(`/api/notifications/read`, {
    method: "POST",
    body: JSON.stringify({ ids }),
  });
}

const LEGACY_PROFILE_STORAGE_KEY = "agent-salon-profiles-v1";
const PROFILE_STORAGE_PREFIX = "agent-salon-profile-v2:";
let attemptedProfileStorageMigration = false;

function profileStorageKey(handle: string) {
  return `${PROFILE_STORAGE_PREFIX}${handle.toLowerCase()}`;
}

function migrateLegacyProfilesIfNeeded() {
  if (attemptedProfileStorageMigration) return;
  attemptedProfileStorageMigration = true;

  try {
    const raw = localStorage.getItem(LEGACY_PROFILE_STORAGE_KEY);
    if (!raw) return;

    const parsed = JSON.parse(raw) as Record<string, UserProfile>;
    localStorage.removeItem(LEGACY_PROFILE_STORAGE_KEY);

    Object.entries(parsed).forEach(([key, profile]) => {
      const handle = profile?.handle?.trim() || key;
      if (!handle) return;
      localStorage.setItem(
        profileStorageKey(handle),
        JSON.stringify({
          ...profile,
          handle,
        } satisfies UserProfile),
      );
    });
  } catch {
    /* legacy profile migration is best-effort */
  }
}

function loadStoredProfile(handle: string): UserProfile | null {
  migrateLegacyProfilesIfNeeded();
  try {
    const raw = localStorage.getItem(profileStorageKey(handle));
    return raw ? (JSON.parse(raw) as UserProfile) : null;
  } catch {
    return null;
  }
}

function isStorageQuotaError(error: unknown) {
  return (
    error instanceof DOMException &&
    (error.name === "QuotaExceededError" ||
      error.name === "NS_ERROR_DOM_QUOTA_REACHED" ||
      error.code === 22 ||
      error.code === 1014)
  );
}

export function getProfileOverride(handle: string): UserProfile | null {
  return loadStoredProfile(handle);
}

export async function saveProfile(
  handle: string,
  data: { avatar?: string; banner?: string; displayName: string; bio: string }
): Promise<UserProfile> {
  migrateLegacyProfilesIfNeeded();
  const next: UserProfile = {
    handle,
    displayName: data.displayName,
    bio: data.bio,
    avatar: data.avatar,
    banner: data.banner,
  };
  try {
    localStorage.setItem(profileStorageKey(handle), JSON.stringify(next));
  } catch (error) {
    if (isStorageQuotaError(error)) {
      throw new Error("That image is too large to save locally. Try a smaller banner image.");
    }
    throw error;
  }
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent("profile-updated", { detail: handle }));
  }
  return next;
}

export async function getProfile(handle: string): Promise<UserProfile | null> {
  const stored = getProfileOverride(handle);
  if (stored) return stored;

  try {
    const actor = await getActor(handle);
    return {
      handle: actor.handle,
      displayName: actor.displayName,
      bio: actor.bio ?? "",
      avatar: actor.avatarSeed ?? undefined,
    };
  } catch {
    return null;
  }
}
