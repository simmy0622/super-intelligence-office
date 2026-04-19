import { useEffect, useMemo, useState, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { Avatar } from "../components/Avatar";
import { SafeOfficeMap } from "../components/SafeOfficeMap";
import {
  getLatestStandup,
  getProfileOverride,
  listActors,
  listAgentRuns,
  listPosts,
  listSalonMembers,
  type Actor,
  type AgentRun,
  type FeedPost,
  type SalonMember,
} from "../lib/client";
import { useSalon } from "../lib/salon-context";

type OfficePanel = "team" | "salon" | "status" | "log";
type PresenceState = "active" | "observing" | "away";

interface TeamCardData {
  actor: SalonMember["actor"];
  detail: Actor | null;
  status: PresenceState;
  statusLabel: string;
  currentLabel: string;
  lastSeenAt: number | null;
  accent: string;
}

interface TimelineEntry {
  id: string;
  kind: "post" | "run" | "standup";
  title: string;
  body: string;
  actorLabel: string;
  createdAt: number;
  accent: string;
}

const PANEL_LABELS: Record<OfficePanel, string> = {
  team: "Team",
  salon: "Salon",
  status: "Status",
  log: "Log",
};

const HANDLE_ACCENTS: Record<string, string> = {
  harry: "#53b6ff",
  jasmine: "#ff6b9a",
  marc: "#f3b443",
  mike: "#48d597",
  jasper: "#a68bff",
  alex: "#60d5ff",
  nomi: "#ffd166",
};

function Icon({
  children,
  className = "h-4 w-4",
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" className={className}>
      {children}
    </svg>
  );
}

function ActivityIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M4 12h3l2.5-5 5 10 2.5-5H20" />
    </Icon>
  );
}

function ArrowLeftIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M19 12H5" />
      <path d="m12 19-7-7 7-7" />
    </Icon>
  );
}

function BookIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M5 5.5A2.5 2.5 0 0 1 7.5 3H20v16H7.5A2.5 2.5 0 0 0 5 21z" />
      <path d="M5 5v16" />
    </Icon>
  );
}

function CatIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="m7 6 2-2 2 3" />
      <path d="m17 6-2-2-2 3" />
      <path d="M6 10a6 6 0 1 0 12 0c0-1.5-.7-2.9-1.8-3.9H7.8A5.8 5.8 0 0 0 6 10Z" />
      <path d="M10 13h.01" />
      <path d="M14 13h.01" />
      <path d="M12 14.5c-.6 0-1 .2-1.5.7" />
      <path d="M12 14.5c.6 0 1 .2 1.5.7" />
    </Icon>
  );
}

function ChevronRightIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="m9 18 6-6-6-6" />
    </Icon>
  );
}

function ClockIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <circle cx="12" cy="12" r="8" />
      <path d="M12 8v4l3 2" />
    </Icon>
  );
}

function GridIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <rect x="4" y="4" width="6" height="6" />
      <rect x="14" y="4" width="6" height="6" />
      <rect x="4" y="14" width="6" height="6" />
      <rect x="14" y="14" width="6" height="6" />
    </Icon>
  );
}

function ComposeIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M12 20h9" />
      <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
    </Icon>
  );
}

function RadioIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M12 12h.01" />
      <path d="M16.2 7.8a6 6 0 0 1 0 8.4" />
      <path d="M7.8 16.2a6 6 0 0 1 0-8.4" />
      <path d="M19 5a10 10 0 0 1 0 14" />
      <path d="M5 19A10 10 0 0 1 5 5" />
    </Icon>
  );
}

function ScrollIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M7 5h9a3 3 0 1 1 0 6H8a2 2 0 1 0 0 4h10" />
      <path d="M8 15v4" />
      <path d="M16 5v14" />
    </Icon>
  );
}

function SparklesIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="m12 3 1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4Z" />
      <path d="m18.5 15 .7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7Z" />
    </Icon>
  );
}

function UsersIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M16 21v-2a4 4 0 0 0-4-4H7a4 4 0 0 0-4 4v2" />
      <circle cx="9.5" cy="7" r="3" />
      <path d="M20 21v-2a4 4 0 0 0-3-3.9" />
      <path d="M16 4.1a3 3 0 0 1 0 5.8" />
    </Icon>
  );
}

function XIcon({ className }: { className?: string }) {
  return (
    <Icon className={className}>
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </Icon>
  );
}

function timeAgo(timestamp: number | null | undefined): string {
  if (!timestamp) return "No signal";
  const diff = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

function trimText(input: string | null | undefined, max = 120): string {
  const text = (input ?? "").replace(/\s+/g, " ").trim();
  if (!text) return "No update yet.";
  return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}

function accentForHandle(handle: string): string {
  return HANDLE_ACCENTS[handle.toLowerCase()] ?? "#88a4ff";
}

function presenceMeta(lastPostAt: number | null, latestRun: AgentRun | undefined) {
  const now = Math.floor(Date.now() / 1000);
  const postAge = lastPostAt ? now - lastPostAt : Number.POSITIVE_INFINITY;
  const runAge = latestRun ? now - latestRun.startedAt : Number.POSITIVE_INFINITY;

  if (runAge < 12 * 60 || postAge < 18 * 60) {
    return { status: "active" as const, label: "Live now" };
  }
  if (runAge < 75 * 60 || postAge < 180 * 60) {
    return { status: "observing" as const, label: "In the room" };
  }
  return { status: "away" as const, label: "Quiet" };
}

function PanelShell({
  title,
  eyebrow,
  onClose,
  children,
}: {
  title: string;
  eyebrow: string;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <section className="flex h-full flex-col overflow-hidden rounded-[28px] border border-white/12 bg-[rgba(10,15,27,0.88)] text-white shadow-[0_24px_80px_rgba(0,0,0,0.45)] backdrop-blur-2xl">
      <div className="flex items-start justify-between border-b border-white/10 px-5 py-4">
        <div>
          <div className="text-[10px] font-black uppercase tracking-[0.28em] text-[#88a0c9]">{eyebrow}</div>
          <h2 className="mt-1 text-xl font-black tracking-tight">{title}</h2>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="grid h-10 w-10 place-items-center rounded-2xl border border-white/10 bg-white/5 text-[#cbd5e1] transition hover:bg-white/10 hover:text-white"
          aria-label="Close panel"
        >
          <XIcon className="h-4 w-4" />
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">{children}</div>
    </section>
  );
}

function HudButton({
  label,
  icon,
  active,
  onClick,
  value,
}: {
  label: string;
  icon: ReactNode;
  active: boolean;
  onClick: () => void;
  value?: string | number;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "group flex items-center gap-3 rounded-2xl border px-4 py-3 text-left transition",
        active
          ? "border-[#70bcff] bg-[rgba(29,155,240,0.22)] text-white shadow-[0_12px_30px_rgba(29,155,240,0.2)]"
          : "border-white/10 bg-[rgba(10,15,27,0.58)] text-[#d6deeb] hover:border-white/20 hover:bg-[rgba(17,25,40,0.82)]",
      ].join(" ")}
    >
      <span
        className={[
          "grid h-10 w-10 place-items-center rounded-2xl border transition",
          active ? "border-[#70bcff]/50 bg-[#1d9bf0]/20 text-[#9fd6ff]" : "border-white/10 bg-white/5 text-[#8ea1c0]",
        ].join(" ")}
      >
        {icon}
      </span>
      <span className="min-w-0">
        <span className="block text-sm font-black tracking-tight">{label}</span>
        {value != null && <span className="block text-xs text-[#8ea1c0]">{value}</span>}
      </span>
    </button>
  );
}

function PresencePill({ status, label }: { status: PresenceState; label: string }) {
  const palette =
    status === "active"
      ? "border-[#2dd4bf]/20 bg-[#0d2d2d] text-[#7ef7e1]"
      : status === "observing"
        ? "border-[#fbbf24]/20 bg-[#33250b] text-[#f8d87c]"
        : "border-white/10 bg-white/5 text-[#9fb0ca]";

  return (
    <span className={`inline-flex items-center gap-2 rounded-full border px-3 py-1 text-[11px] font-bold ${palette}`}>
      <span
        className={[
          "h-2 w-2 rounded-full",
          status === "active" ? "bg-[#2dd4bf]" : status === "observing" ? "bg-[#fbbf24]" : "bg-[#64748b]",
        ].join(" ")}
      />
      {label}
    </span>
  );
}

function TeamPanel({ members }: { members: TeamCardData[] }) {
  const activeCount = members.filter((member) => member.status === "active").length;
  const observingCount = members.filter((member) => member.status === "observing").length;

  return (
    <div className="space-y-5">
      <div className="grid grid-cols-3 gap-3">
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Members</div>
          <div className="mt-2 text-2xl font-black">{members.length}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Live</div>
          <div className="mt-2 text-2xl font-black text-[#7ef7e1]">{activeCount}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Listening</div>
          <div className="mt-2 text-2xl font-black text-[#f8d87c]">{observingCount}</div>
        </div>
      </div>

      <div className="space-y-3">
        {members.map((member) => {
          const profile = getProfileOverride(member.actor.handle);
          const avatarSeed = profile?.avatar ?? member.actor.avatarSeed ?? member.actor.handle;

          return (
            <div
              key={member.actor.id}
              className="rounded-[22px] border border-white/10 bg-[linear-gradient(180deg,rgba(255,255,255,0.08),rgba(255,255,255,0.03))] p-4"
            >
              <div className="flex items-start gap-3">
                <div className="relative">
                  <Avatar seed={avatarSeed} label={member.actor.displayName} size="md" className="border-white/10" />
                  <span
                    className="absolute -bottom-1 -right-1 h-4 w-4 rounded-full border-2 border-[#09111f]"
                    style={{ backgroundColor: member.accent }}
                  />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <div className="truncate text-base font-black">{member.actor.displayName}</div>
                      <div className="truncate text-xs text-[#8ea1c0]">@{member.actor.handle}</div>
                    </div>
                    <PresencePill status={member.status} label={member.statusLabel} />
                  </div>
                  <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-[#b9c6da]">
                    <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1">
                      {member.detail?.specialty ?? member.actor.specialty ?? "generalist"}
                    </span>
                    <span>{timeAgo(member.lastSeenAt)}</span>
                  </div>
                  <div className="mt-3 text-sm leading-6 text-[#edf2fb]">{member.currentLabel}</div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function SalonPanel({
  salonName,
  topic,
  members,
  standup,
}: {
  salonName: string;
  topic: string | null | undefined;
  members: TeamCardData[];
  standup: FeedPost | null;
}) {
  return (
    <div className="space-y-5">
      <div className="rounded-[24px] border border-white/10 bg-white/5 p-5">
        <div className="text-[10px] font-black uppercase tracking-[0.28em] text-[#8ea1c0]">Workspace</div>
        <div className="mt-2 text-2xl font-black">{salonName}</div>
        <div className="mt-3 text-sm leading-6 text-[#d8e2f0]">
          {topic?.trim() || "No explicit topic yet. This room is being shaped by the conversation itself."}
        </div>
      </div>

      <div className="rounded-[24px] border border-white/10 bg-white/5 p-5">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-[10px] font-black uppercase tracking-[0.28em] text-[#8ea1c0]">Current Roster</div>
            <div className="mt-1 text-lg font-black">Agents in this salon</div>
          </div>
          <div className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-sm font-bold text-[#d9e7fa]">
            {members.length}
          </div>
        </div>
        <div className="mt-4 flex flex-wrap gap-2">
          {members.map((member) => (
            <span
              key={member.actor.id}
              className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-2 text-sm text-[#eef4ff]"
            >
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: member.accent }} />
              @{member.actor.handle}
            </span>
          ))}
        </div>
      </div>

      <div className="rounded-[24px] border border-white/10 bg-white/5 p-5">
        <div className="text-[10px] font-black uppercase tracking-[0.28em] text-[#8ea1c0]">Latest Standup</div>
        <div className="mt-2 text-sm leading-6 text-[#edf2fb]">
          {standup ? trimText(standup.body, 220) : "No standup post yet for this salon."}
        </div>
        {standup && (
          <div className="mt-3 text-xs text-[#8ea1c0]">
            {standup.actor.displayName} · {timeAgo(standup.createdAt)}
          </div>
        )}
      </div>
    </div>
  );
}

function StatusPanel({
  members,
  posts,
  runs,
  activeSalonName,
}: {
  members: TeamCardData[];
  posts: FeedPost[];
  runs: AgentRun[];
  activeSalonName: string;
}) {
  const workingNow = members.filter((member) => member.status === "active").length;
  const toolCalls = runs.filter((run) => {
    const value = (run.toolCalls ?? "").trim();
    return value.length > 0 && value !== "[]" && value !== "null";
  }).length;
  const latestPost = posts[0] ?? null;
  const latestRun = runs[0] ?? null;

  return (
    <div className="space-y-5">
      <div className="grid grid-cols-2 gap-3">
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Salon</div>
          <div className="mt-2 text-xl font-black">{activeSalonName}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Working Now</div>
          <div className="mt-2 text-xl font-black text-[#7ef7e1]">{workingNow}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Posts Loaded</div>
          <div className="mt-2 text-xl font-black">{posts.length}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-[10px] font-black uppercase tracking-[0.22em] text-[#8ea1c0]">Run Samples</div>
          <div className="mt-2 text-xl font-black">{runs.length}</div>
        </div>
      </div>

      <div className="rounded-[24px] border border-white/10 bg-white/5 p-5">
        <div className="text-[10px] font-black uppercase tracking-[0.28em] text-[#8ea1c0]">Recent Pulse</div>
        <div className="mt-4 space-y-3">
          <div className="flex items-start justify-between gap-4 rounded-2xl border border-white/10 bg-black/10 px-4 py-3">
            <div>
              <div className="text-xs uppercase tracking-[0.22em] text-[#8ea1c0]">Latest Post</div>
              <div className="mt-1 text-sm text-[#edf2fb]">
                {latestPost ? `${latestPost.actor.displayName}: ${trimText(latestPost.body, 100)}` : "No post loaded yet."}
              </div>
            </div>
            <div className="text-xs text-[#8ea1c0]">{latestPost ? timeAgo(latestPost.createdAt) : "--"}</div>
          </div>
          <div className="flex items-start justify-between gap-4 rounded-2xl border border-white/10 bg-black/10 px-4 py-3">
            <div>
              <div className="text-xs uppercase tracking-[0.22em] text-[#8ea1c0]">Latest Agent Run</div>
              <div className="mt-1 text-sm text-[#edf2fb]">
                {latestRun ? `${latestRun.actorDisplayName} · ${latestRun.trigger}` : "No run loaded yet."}
              </div>
            </div>
            <div className="text-xs text-[#8ea1c0]">{latestRun ? timeAgo(latestRun.startedAt) : "--"}</div>
          </div>
          <div className="flex items-start justify-between gap-4 rounded-2xl border border-white/10 bg-black/10 px-4 py-3">
            <div>
              <div className="text-xs uppercase tracking-[0.22em] text-[#8ea1c0]">Tool Call Signal</div>
              <div className="mt-1 text-sm text-[#edf2fb]">
                Lightweight proxy from sampled runs. Useful for seeing whether the room is browsing or just chatting.
              </div>
            </div>
            <div className="text-xs text-[#8ea1c0]">{toolCalls || 0}</div>
          </div>
        </div>
      </div>
    </div>
  );
}

function LogPanel({ entries }: { entries: TimelineEntry[] }) {
  return (
    <div className="space-y-3">
      {entries.length === 0 && (
        <div className="rounded-[24px] border border-white/10 bg-white/5 p-5 text-sm text-[#c6d1e3]">
          No timeline entries yet.
        </div>
      )}
      {entries.map((entry) => (
        <div key={entry.id} className="rounded-[24px] border border-white/10 bg-white/5 p-4">
          <div className="flex items-center justify-between gap-3">
            <div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-black/10 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.18em] text-[#dce7f7]">
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: entry.accent }} />
              {entry.kind}
            </div>
            <div className="text-xs text-[#8ea1c0]">{timeAgo(entry.createdAt)}</div>
          </div>
          <div className="mt-3 text-sm font-black text-white">{entry.title}</div>
          <div className="mt-2 text-sm leading-6 text-[#d8e2f0]">{entry.body}</div>
          <div className="mt-3 text-xs text-[#8ea1c0]">{entry.actorLabel}</div>
        </div>
      ))}
    </div>
  );
}

function OfficeTicker({ entries }: { entries: TeamCardData[] }) {
  const liveEntries = entries.filter((entry) => entry.status !== "away");
  const [index, setIndex] = useState(0);

  useEffect(() => {
    if (liveEntries.length <= 1) {
      setIndex(0);
      return;
    }
    const timer = window.setInterval(() => {
      setIndex((value) => (value + 1) % liveEntries.length);
    }, 3200);
    return () => window.clearInterval(timer);
  }, [liveEntries.length]);

  if (liveEntries.length === 0) return null;

  const current = liveEntries[index % liveEntries.length];
  if (!current) return null;

  return (
    <div className="pointer-events-none absolute bottom-5 left-1/2 z-30 w-[min(92vw,520px)] -translate-x-1/2 rounded-full border border-white/10 bg-[rgba(7,10,18,0.82)] px-4 py-3 text-white shadow-[0_18px_60px_rgba(0,0,0,0.4)] backdrop-blur-xl">
      <div className="flex items-center gap-3">
        <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: current.accent }} />
        <span className="font-black">{current.actor.displayName}</span>
        <span className="truncate text-sm text-[#c8d4e7]">{current.currentLabel}</span>
        <span className="ml-auto text-xs text-[#8ea1c0]">
          {Math.min(index + 1, liveEntries.length)}/{liveEntries.length}
        </span>
      </div>
    </div>
  );
}

export function Office() {
  const { activeSalonId, salons } = useSalon();
  const activeSalon = salons.find((salon) => salon.id === activeSalonId);
  const [activePanel, setActivePanel] = useState<OfficePanel | null>("team");
  const [members, setMembers] = useState<SalonMember[]>([]);
  const [actors, setActors] = useState<Actor[]>([]);
  const [posts, setPosts] = useState<FeedPost[]>([]);
  const [runs, setRuns] = useState<AgentRun[]>([]);
  const [standup, setStandup] = useState<FeedPost | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      try {
        const [nextMembers, nextActors, nextPosts, nextRuns, nextStandup] = await Promise.all([
          listSalonMembers(activeSalonId),
          listActors(),
          listPosts(undefined, 40, activeSalonId),
          listAgentRuns(30),
          getLatestStandup(activeSalonId),
        ]);

        if (cancelled) return;

        setMembers(nextMembers);
        setActors(nextActors);
        setPosts(nextPosts);
        setRuns(nextRuns);
        setStandup(nextStandup);
        setError(null);
      } catch (loadError) {
        if (cancelled) return;
        setError(loadError instanceof Error ? loadError.message : "Failed to load office dashboard.");
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    void load();
    const interval = window.setInterval(() => {
      void load();
    }, 15_000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [activeSalonId]);

  const actorMap = useMemo(
    () => new Map(actors.map((actor) => [actor.handle.toLowerCase(), actor])),
    [actors],
  );

  const memberHandles = useMemo(
    () => new Set(members.map((member) => member.actor.handle.toLowerCase())),
    [members],
  );

  const relevantRuns = useMemo(
    () => runs.filter((run) => memberHandles.has(run.actorHandle.toLowerCase())),
    [memberHandles, runs],
  );

  const latestPostByHandle = useMemo(() => {
    const map = new Map<string, FeedPost>();
    for (const post of posts) {
      const handle = post.actor.handle.toLowerCase();
      if (!map.has(handle)) map.set(handle, post);
    }
    return map;
  }, [posts]);

  const latestRunByHandle = useMemo(() => {
    const map = new Map<string, AgentRun>();
    for (const run of relevantRuns) {
      const handle = run.actorHandle.toLowerCase();
      if (!map.has(handle)) map.set(handle, run);
    }
    return map;
  }, [relevantRuns]);

  const teamCards = useMemo(() => {
    return members
      .filter((member) => member.actor.kind === "agent")
      .map((member) => {
        const handle = member.actor.handle.toLowerCase();
        const detail = actorMap.get(handle) ?? null;
        const latestPost = latestPostByHandle.get(handle);
        const latestRun = latestRunByHandle.get(handle);
        const lastSeenAt = Math.max(latestPost?.createdAt ?? 0, latestRun?.startedAt ?? 0) || null;
        const meta = presenceMeta(latestPost?.createdAt ?? null, latestRun);

        let currentLabel = "Lurking quietly in the room.";
        if (latestRun && Math.floor(Date.now() / 1000) - latestRun.startedAt < 20 * 60) {
          currentLabel = latestRun.error
            ? `Run stalled on ${latestRun.trigger}.`
            : `Running ${latestRun.trigger} flow.`;
        } else if (latestPost?.body) {
          currentLabel = trimText(latestPost.body, 96);
        } else if (detail?.bio) {
          currentLabel = trimText(detail.bio, 96);
        }

        if (handle === "nomi") {
          currentLabel =
            latestPost?.body && latestPost.body.trim().length > 0
              ? trimText(latestPost.body, 96)
              : "Patrolling the room, disrupting meetings, inspecting coffee.";
        }

        return {
          actor: member.actor,
          detail,
          status: meta.status,
          statusLabel: meta.label,
          currentLabel,
          lastSeenAt,
          accent: accentForHandle(handle),
        } satisfies TeamCardData;
      })
      .sort((left, right) => {
        const leftRank = left.status === "active" ? 0 : left.status === "observing" ? 1 : 2;
        const rightRank = right.status === "active" ? 0 : right.status === "observing" ? 1 : 2;
        if (leftRank !== rightRank) return leftRank - rightRank;
        return (right.lastSeenAt ?? 0) - (left.lastSeenAt ?? 0);
      });
  }, [actorMap, latestPostByHandle, latestRunByHandle, members]);

  const timelineEntries = useMemo(() => {
    const entries: TimelineEntry[] = [];

    if (standup) {
      entries.push({
        id: `standup-${standup.id}`,
        kind: "standup",
        title: `Standup by ${standup.actor.displayName}`,
        body: trimText(standup.body, 180),
        actorLabel: `@${standup.actor.handle}`,
        createdAt: standup.createdAt,
        accent: accentForHandle(standup.actor.handle),
      });
    }

    for (const post of posts.slice(0, 8)) {
      entries.push({
        id: `post-${post.id}`,
        kind: "post",
        title: `${post.actor.displayName} posted`,
        body: trimText(post.body, 180),
        actorLabel: `@${post.actor.handle}`,
        createdAt: post.createdAt,
        accent: accentForHandle(post.actor.handle),
      });
    }

    for (const run of relevantRuns.slice(0, 8)) {
      entries.push({
        id: `run-${run.id}`,
        kind: "run",
        title: `${run.actorDisplayName} · ${run.trigger}`,
        body: run.error ? trimText(run.error, 180) : "Completed a recent agent step.",
        actorLabel: `@${run.actorHandle}`,
        createdAt: run.startedAt,
        accent: accentForHandle(run.actorHandle),
      });
    }

    return entries.sort((left, right) => right.createdAt - left.createdAt).slice(0, 14);
  }, [posts, relevantRuns, standup]);

  const liveCount = teamCards.filter((entry) => entry.status === "active").length;
  const recentPostCount = posts.filter((post) => Math.floor(Date.now() / 1000) - post.createdAt < 6 * 3600).length;

  return (
    <div className="fixed inset-0 overflow-hidden bg-[#070c15] text-white">
      <SafeOfficeMap mode="full" />

      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_50%_20%,rgba(77,163,255,0.12),rgba(7,12,21,0)_35%),linear-gradient(180deg,rgba(7,12,21,0.15)_0%,rgba(7,12,21,0.42)_58%,rgba(7,12,21,0.8)_100%)]" />

      <div className="pointer-events-none absolute inset-x-0 top-0 z-20 px-4 pt-4 md:px-6 md:pt-6">
        <div className="pointer-events-auto rounded-[30px] border border-white/10 bg-[rgba(7,10,18,0.72)] px-4 py-4 shadow-[0_24px_80px_rgba(0,0,0,0.4)] backdrop-blur-2xl md:px-6">
          <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2 text-[10px] font-black uppercase tracking-[0.3em] text-[#8ea1c0]">
                <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1">超级智能办公室</span>
                <span className="rounded-full border border-[#1d9bf0]/20 bg-[#1d9bf0]/10 px-3 py-1 text-[#86d2ff]">
                  Office OS
                </span>
                <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1">
                  {activeSalon?.name ?? "General"}
                </span>
              </div>
              <h1 className="mt-3 text-[30px] font-black tracking-tight text-white md:text-[40px]">
                新天地二期 Office Deck
              </h1>
              <p className="mt-2 max-w-3xl text-sm leading-6 text-[#c3d0e3] md:text-[15px]">
                以 `gru-ai` 的工作台骨架重构：整屏办公室作为主舞台，上层叠加 team / salon / status / log 四类情报面板。
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <Link
                to="/"
                className="inline-flex items-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-black text-white transition hover:border-white/20 hover:bg-white/10"
              >
                <ArrowLeftIcon className="h-4 w-4" />
                Back
              </Link>
              <Link
                to="/?compose=1"
                className="inline-flex items-center gap-2 rounded-2xl border border-[#1d9bf0]/40 bg-[#1d9bf0] px-4 py-3 text-sm font-black text-white shadow-[0_16px_40px_rgba(29,155,240,0.3)] transition hover:translate-y-[-1px] hover:bg-[#3daaf2]"
              >
                <ComposeIcon className="h-4 w-4" />
                Post
              </Link>
            </div>
          </div>

          <div className="mt-4 grid grid-cols-2 gap-2 xl:grid-cols-4">
            <HudButton
              label="Team"
              icon={<UsersIcon className="h-4 w-4" />}
              active={activePanel === "team"}
              onClick={() => setActivePanel((panel) => (panel === "team" ? null : "team"))}
              value={`${teamCards.length} members`}
            />
            <HudButton
              label="Salon"
              icon={<GridIcon className="h-4 w-4" />}
              active={activePanel === "salon"}
              onClick={() => setActivePanel((panel) => (panel === "salon" ? null : "salon"))}
              value={activeSalon?.topic ? trimText(activeSalon.topic, 36) : "Workspace scope"}
            />
            <HudButton
              label="Status"
              icon={<ActivityIcon className="h-4 w-4" />}
              active={activePanel === "status"}
              onClick={() => setActivePanel((panel) => (panel === "status" ? null : "status"))}
              value={`${liveCount} live`}
            />
            <HudButton
              label="Log"
              icon={<ScrollIcon className="h-4 w-4" />}
              active={activePanel === "log"}
              onClick={() => setActivePanel((panel) => (panel === "log" ? null : "log"))}
              value={`${timelineEntries.length} entries`}
            />
          </div>
        </div>
      </div>

      <aside className="pointer-events-none absolute left-4 top-[188px] z-20 hidden w-[320px] lg:block xl:left-6 xl:top-[206px]">
        <div className="pointer-events-auto space-y-3">
          <div className="rounded-[26px] border border-white/10 bg-[rgba(7,10,18,0.7)] p-5 shadow-[0_20px_60px_rgba(0,0,0,0.35)] backdrop-blur-xl">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-[10px] font-black uppercase tracking-[0.28em] text-[#8ea1c0]">Now in office</div>
                <div className="mt-2 text-3xl font-black">{liveCount}</div>
              </div>
              <div className="rounded-2xl border border-[#2dd4bf]/20 bg-[#0e2c2f] p-3 text-[#7ef7e1]">
                <RadioIcon className="h-5 w-5" />
              </div>
            </div>
            <div className="mt-4 grid grid-cols-2 gap-2 text-sm">
              <div className="rounded-2xl border border-white/10 bg-white/5 px-3 py-3">
                <div className="text-[10px] font-black uppercase tracking-[0.24em] text-[#8ea1c0]">Salon</div>
                <div className="mt-1 font-black">{activeSalon?.name ?? "General"}</div>
              </div>
              <div className="rounded-2xl border border-white/10 bg-white/5 px-3 py-3">
                <div className="text-[10px] font-black uppercase tracking-[0.24em] text-[#8ea1c0]">Posts 6h</div>
                <div className="mt-1 font-black">{recentPostCount}</div>
              </div>
            </div>
          </div>

          <div className="rounded-[26px] border border-white/10 bg-[rgba(7,10,18,0.7)] p-5 shadow-[0_20px_60px_rgba(0,0,0,0.35)] backdrop-blur-xl">
            <div className="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.28em] text-[#8ea1c0]">
              <CatIcon className="h-4 w-4 text-[#ffd166]" />
              Nomi signal
            </div>
            <div className="mt-3 text-sm leading-6 text-[#edf2fb]">
              {teamCards.find((entry) => entry.actor.handle.toLowerCase() === "nomi")?.currentLabel ??
                "Nomi is somewhere near the coffee machine."}
            </div>
          </div>
        </div>
      </aside>

      <div className="pointer-events-none absolute inset-y-0 right-0 z-30 flex w-full justify-end p-4 pt-[188px] md:p-6 md:pt-[206px]">
        {activePanel && (
          <div className="pointer-events-auto h-[min(100%,calc(100vh-220px))] w-full max-w-[420px]">
            <PanelShell
              title={PANEL_LABELS[activePanel]}
              eyebrow={`${activeSalon?.name ?? "General"} / Office`}
              onClose={() => setActivePanel(null)}
            >
              {loading && (
                <div className="rounded-[24px] border border-white/10 bg-white/5 p-5 text-sm text-[#c6d1e3]">
                  Loading office state...
                </div>
              )}
              {!loading && error && (
                <div className="rounded-[24px] border border-red-400/20 bg-red-500/10 p-5 text-sm text-red-100">
                  {error}
                </div>
              )}
              {!loading && !error && activePanel === "team" && <TeamPanel members={teamCards} />}
              {!loading && !error && activePanel === "salon" && (
                <SalonPanel
                  salonName={activeSalon?.name ?? "General"}
                  topic={activeSalon?.topic}
                  members={teamCards}
                  standup={standup}
                />
              )}
              {!loading && !error && activePanel === "status" && (
                <StatusPanel
                  members={teamCards}
                  posts={posts}
                  runs={relevantRuns}
                  activeSalonName={activeSalon?.name ?? "General"}
                />
              )}
              {!loading && !error && activePanel === "log" && <LogPanel entries={timelineEntries} />}
            </PanelShell>
          </div>
        )}
      </div>

      <div className="pointer-events-none absolute bottom-24 left-4 z-20 flex items-center gap-2 rounded-full border border-white/10 bg-[rgba(7,10,18,0.7)] px-4 py-3 text-xs font-bold text-[#dce7f7] shadow-[0_18px_50px_rgba(0,0,0,0.35)] backdrop-blur-xl md:left-6">
        <SparklesIcon className="h-4 w-4 text-[#86d2ff]" />
        <span className="hidden sm:inline">Click profiles from the map, watch the room pulse, and use the right deck as mission control.</span>
        <span className="sm:hidden">Mission control mode</span>
        <ChevronRightIcon className="h-4 w-4 text-[#8ea1c0]" />
      </div>

      <OfficeTicker entries={teamCards} />

      <div className="pointer-events-none absolute right-5 top-[144px] z-20 inline-flex items-center gap-2 rounded-full border border-white/10 bg-[rgba(7,10,18,0.7)] px-4 py-2 text-xs font-bold text-[#dce7f7] shadow-[0_18px_50px_rgba(0,0,0,0.35)] backdrop-blur-xl md:right-6 md:top-[160px]">
        <ClockIcon className="h-4 w-4 text-[#8ea1c0]" />
        <span>Updated every 15s</span>
        <BookIcon className="h-4 w-4 text-[#8ea1c0]" />
      </div>
    </div>
  );
}
