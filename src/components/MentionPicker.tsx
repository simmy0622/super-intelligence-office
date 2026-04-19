import { Avatar } from "./Avatar";
import type { Actor } from "../lib/client";

interface MentionPickerProps {
  agents: Actor[];
  query: string;
  onSelect: (handle: string) => void;
}

export function MentionPicker({ agents, query, onSelect }: MentionPickerProps) {
  const normalizedQuery = query.trim().toLowerCase();
  const matches = agents
    .filter((agent) => {
      if (!normalizedQuery) return true;
      return (
        agent.handle.toLowerCase().includes(normalizedQuery) ||
        agent.displayName.toLowerCase().includes(normalizedQuery)
      );
    })
    .slice(0, 8);

  if (matches.length === 0) return null;

  return (
    <div className="absolute left-0 top-full z-50 mt-2 w-72 overflow-hidden rounded-2xl border border-x-border bg-white shadow-xl shadow-black/10 dark:border-x-border-dark dark:bg-black">
      <div className="border-b border-x-border px-4 py-2 text-xs font-semibold uppercase tracking-wide text-x-text-secondary dark:border-x-border-dark">
        Mention an agent
      </div>
      <div className="max-h-80 overflow-y-auto py-1">
        {matches.map((agent) => (
          <button
            key={agent.id}
            type="button"
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => onSelect(agent.handle)}
            className="flex w-full items-center gap-3 px-4 py-2.5 text-left transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
          >
            <Avatar seed={agent.avatarSeed ?? agent.handle} label={agent.displayName} size="xs" />
            <div className="min-w-0">
              <div className="truncate text-sm font-bold text-x-text dark:text-x-text-dark">
                {agent.displayName}
              </div>
              <div className="truncate text-xs text-x-text-secondary">
                @{agent.handle}
                {agent.specialty ? ` · ${agent.specialty}` : ""}
              </div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
