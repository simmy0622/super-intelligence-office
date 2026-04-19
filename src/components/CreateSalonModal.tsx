import { FormEvent, useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import { createSalon, listActors, type Actor, type Salon } from "../lib/client";
import { cn } from "../lib/utils";
import { Avatar } from "./Avatar";

interface CreateSalonModalProps {
  open: boolean;
  onClose: () => void;
  onCreated: (salon: Salon) => void;
}

function isNomi(handle: string) {
  return handle.toLowerCase() === "nomi" || handle.toLowerCase() === "nuomi";
}

export function CreateSalonModal({ open, onClose, onCreated }: CreateSalonModalProps) {
  const [name, setName] = useState("");
  const [topic, setTopic] = useState("");
  const [actors, setActors] = useState<Actor[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setError(null);
    void listActors()
      .then((next) => {
        setActors(next);
        setSelectedIds(
          new Set(
            next
              .filter((actor) => actor.kind === "agent" && !isNomi(actor.handle))
              .map((actor) => actor.id),
          ),
        );
      })
      .catch((nextError) => {
        setError(nextError instanceof Error ? nextError.message : "Failed to load agents.");
      });
  }, [open]);

  const human = useMemo(() => actors.find((actor) => actor.kind === "human"), [actors]);
  const agents = useMemo(
    () => actors.filter((actor) => actor.kind === "agent" && !isNomi(actor.handle)),
    [actors],
  );

  if (!open || typeof document === "undefined") return null;

  const toggleAgent = (id: number) => {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!human || !name.trim()) return;
    setSubmitting(true);
    setError(null);
    try {
      const salon = await createSalon({
        name: name.trim(),
        topic: topic.trim() || null,
        createdBy: human.id,
        memberActorIds: [...selectedIds],
      });
      setName("");
      setTopic("");
      onCreated(salon);
      onClose();
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to create salon.");
    } finally {
      setSubmitting(false);
    }
  };

  return createPortal(
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/45 px-4 py-6 backdrop-blur-md">
      <form
        onSubmit={handleSubmit}
        className="flex max-h-[88vh] w-full max-w-[560px] flex-col overflow-hidden rounded-[2rem] border border-x-border bg-x-background shadow-[0_24px_80px_rgba(15,20,25,0.28)] dark:border-x-border-dark dark:bg-x-background-dark"
      >
        <div className="border-b border-x-border px-5 py-4 dark:border-x-border-dark">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <p className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-primary">
                New workspace
              </p>
              <h2 className="mt-1 text-2xl font-black tracking-[-0.03em] text-x-text dark:text-x-text-dark">
                Create salon
              </h2>
              <p className="mt-1 max-w-[36ch] text-sm leading-5 text-x-text-secondary">
                Create a focused room, invite agents, and keep the conversation scoped.
              </p>
            </div>
            <button
              type="button"
              onClick={onClose}
              className="shrink-0 rounded-full border border-x-border px-3 py-1.5 text-sm font-bold text-x-text-secondary transition-colors hover:bg-x-surface-hover dark:border-x-border-dark dark:hover:bg-x-surface-hover-dark"
            >
              Close
            </button>
          </div>
        </div>

        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-5">
          <div className="grid gap-4">
            <label className="block text-sm font-bold text-x-text dark:text-x-text-dark">
              Name
              <input
                value={name}
                onChange={(event) => setName(event.target.value)}
                maxLength={60}
                className="mt-2 w-full rounded-2xl border border-x-border bg-white px-4 py-3 text-base text-x-text outline-none transition-colors placeholder:text-x-text-secondary/70 focus:border-x-primary focus:ring-4 focus:ring-x-primary/10 dark:border-x-border-dark dark:bg-black dark:text-x-text-dark"
                placeholder="AI Investing Room"
              />
            </label>

            <label className="block text-sm font-bold text-x-text dark:text-x-text-dark">
              Topic
              <textarea
                value={topic}
                onChange={(event) => setTopic(event.target.value)}
                rows={3}
                className="mt-2 w-full resize-none rounded-2xl border border-x-border bg-white px-4 py-3 text-base leading-6 text-x-text outline-none transition-colors placeholder:text-x-text-secondary/70 focus:border-x-primary focus:ring-4 focus:ring-x-primary/10 dark:border-x-border-dark dark:bg-black dark:text-x-text-dark"
                placeholder="AI models, capital markets, product strategy..."
              />
            </label>
          </div>

          <div className="mt-5 rounded-3xl border border-x-border bg-x-surface-hover/40 p-3 dark:border-x-border-dark dark:bg-x-surface-hover-dark/40">
            <div className="flex items-center justify-between gap-3 px-1">
              <div>
                <div className="text-sm font-black text-x-text dark:text-x-text-dark">Invite agents</div>
                <p className="mt-0.5 text-xs leading-4 text-x-text-secondary">
                  Nomi joins every salon automatically.
                </p>
              </div>
              <span className="shrink-0 rounded-full bg-x-primary/10 px-2.5 py-1 text-xs font-extrabold text-x-primary">
                {selectedIds.size}/{agents.length}
              </span>
            </div>

            <div className="mt-3 grid grid-cols-1 gap-2 sm:grid-cols-2">
              {agents.map((agent) => {
                const selected = selectedIds.has(agent.id);
                return (
                  <button
                    key={agent.id}
                    type="button"
                    onClick={() => toggleAgent(agent.id)}
                    className={cn(
                      "group flex min-h-[68px] items-center gap-3 rounded-2xl border px-3 py-2 text-left transition-all",
                      selected
                        ? "border-x-primary bg-x-primary text-white shadow-[0_8px_24px_rgba(29,155,240,0.22)]"
                        : "border-x-border bg-white text-x-text hover:border-x-primary/40 hover:bg-x-surface-hover dark:border-x-border-dark dark:bg-black dark:text-x-text-dark dark:hover:bg-x-surface-hover-dark",
                    )}
                    aria-pressed={selected}
                  >
                    <Avatar
                      seed={agent.avatarSeed}
                      label={agent.displayName}
                      size="sm"
                      className={cn(selected && "border-white/40")}
                    />
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-black">@{agent.handle}</div>
                      <div
                        className={cn(
                          "mt-0.5 line-clamp-1 text-xs",
                          selected ? "text-white/80" : "text-x-text-secondary",
                        )}
                      >
                        {agent.specialty ?? agent.bio ?? "Agent"}
                      </div>
                    </div>
                    <span
                      className={cn(
                        "flex h-5 w-5 shrink-0 items-center justify-center rounded-full border text-[11px] font-black",
                        selected
                          ? "border-white bg-white text-x-primary"
                          : "border-x-border text-transparent group-hover:border-x-primary dark:border-x-border-dark",
                      )}
                    >
                      ✓
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          {error && (
            <p className="mt-4 rounded-2xl bg-red-500/10 px-4 py-3 text-sm font-medium text-red-600 dark:text-red-400">
              {error}
            </p>
          )}
        </div>

        <div className="flex items-center gap-3 border-t border-x-border bg-x-background px-5 py-4 dark:border-x-border-dark dark:bg-x-background-dark">
          <p className="min-w-0 flex-1 text-xs leading-4 text-x-text-secondary">
            V1 supports up to 3 persistent salons.
          </p>
          <button
            type="submit"
            disabled={!name.trim() || !human || submitting}
            className="shrink-0 rounded-full bg-x-primary px-5 py-3 text-sm font-extrabold text-white transition-colors hover:bg-x-primary-hover disabled:cursor-not-allowed disabled:opacity-50"
          >
            {submitting ? "Creating..." : "Create salon"}
          </button>
        </div>
      </form>
    </div>,
    document.body,
  );
}
