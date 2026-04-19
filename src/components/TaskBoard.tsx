import { useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import { Link } from "react-router-dom";

import { cn } from "../lib/utils";
import {
  createTask,
  deleteTask,
  listActors,
  listTasks,
  reopenTask,
  updateTask,
  type Actor,
  type Task,
} from "../lib/client";

interface TaskBoardProps {
  salonId: number;
  collapsed?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

type TaskStatus = Task["status"];

interface TaskColumnProps {
  title: string;
  status: TaskStatus;
  tasks: Task[];
  agents: Actor[];
  humanActorId: number | null;
  onRefresh: () => Promise<void>;
  onCreate?: (title: string, description?: string, assignedToHandle?: string) => Promise<void>;
}

const columns: Array<{ key: TaskStatus; title: string }> = [
  { key: "todo", title: "Todo" },
  { key: "in_progress", title: "In Progress" },
  { key: "done", title: "Done" },
];

function BoardIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="currentColor">
      <path d="M4 4h7v7H4V4Zm9 0h7v5h-7V4ZM4 13h5v7H4v-7Zm7 0h9v7h-9v-7Z" />
    </svg>
  );
}

function LinkIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="currentColor">
      <path d="M18.36 5.64a5 5 0 0 0-7.07 0l-1.65 1.65 1.41 1.41 1.65-1.65a3 3 0 1 1 4.24 4.24l-2.12 2.12 1.41 1.41 2.12-2.12a5 5 0 0 0 0-7.06ZM12.95 15.29l-1.41-1.41-1.65 1.65a3 3 0 1 1-4.24-4.24l2.12-2.12-1.41-1.41-2.12 2.12a5 5 0 1 0 7.07 7.07l1.64-1.66Zm2.12-6.36-5.66 5.66-1.41-1.41 5.66-5.66 1.41 1.41Z" />
    </svg>
  );
}

function countByStatus(tasks: Task[], status: TaskStatus) {
  return tasks.filter((task) => task.status === status).length;
}

function TaskCard({
  task,
  agents,
  humanActorId,
  onRefresh,
}: {
  task: Task;
  agents: Actor[];
  humanActorId: number | null;
  onRefresh: () => Promise<void>;
}) {
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description ?? "");
  const [assignedToHandle, setAssignedToHandle] = useState(task.assignedToHandle ?? "");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    setTitle(task.title);
    setDescription(task.description ?? "");
    setAssignedToHandle(task.assignedToHandle ?? "");
  }, [task]);

  const save = async () => {
    setBusy(true);
    try {
      await updateTask(task.id, {
        title,
        description,
        assignedToHandle: assignedToHandle || null,
      });
      setEditing(false);
      await onRefresh();
    } finally {
      setBusy(false);
    }
  };

  const quickMove = async (status: TaskStatus) => {
    setBusy(true);
    try {
      await updateTask(task.id, { status });
      await onRefresh();
    } finally {
      setBusy(false);
    }
  };

  const reopen = async () => {
    if (!humanActorId) return;
    setBusy(true);
    try {
      await reopenTask(task.id, humanActorId);
      await onRefresh();
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (!window.confirm(`Delete task "${task.title}"?`)) return;
    setBusy(true);
    try {
      await deleteTask(task.id);
      await onRefresh();
    } finally {
      setBusy(false);
    }
  };

  return (
    <article className="rounded-2xl border border-x-border bg-white p-3 shadow-sm dark:border-x-border-dark dark:bg-[#0f1722]">
      {editing ? (
        <div className="space-y-2">
          <input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            className="w-full rounded-xl border border-x-border bg-transparent px-3 py-2 text-sm font-semibold outline-none dark:border-x-border-dark"
            placeholder="Task title"
          />
          <textarea
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            className="min-h-[76px] w-full rounded-xl border border-x-border bg-transparent px-3 py-2 text-sm outline-none dark:border-x-border-dark"
            placeholder="Description"
          />
          <select
            value={assignedToHandle}
            onChange={(event) => setAssignedToHandle(event.target.value)}
            className="w-full rounded-xl border border-x-border bg-transparent px-3 py-2 text-sm outline-none dark:border-x-border-dark"
          >
            <option value="">Unassigned</option>
            {agents.map((agent) => (
              <option key={agent.id} value={agent.handle}>
                @{agent.handle}
              </option>
            ))}
          </select>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={save}
              disabled={busy}
              className="rounded-full bg-x-primary px-3 py-1.5 text-xs font-bold text-white disabled:opacity-50"
            >
              Save
            </button>
            <button
              type="button"
              onClick={() => setEditing(false)}
              className="rounded-full border border-x-border px-3 py-1.5 text-xs font-bold dark:border-x-border-dark"
            >
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <>
          <div className="flex items-start gap-2">
            <div className="min-w-0 flex-1">
              <h4 className="line-clamp-2 text-sm font-bold text-x-text dark:text-x-text-dark">{task.title}</h4>
              {task.description && (
                <p className="mt-1 line-clamp-2 text-xs leading-5 text-x-text-secondary">{task.description}</p>
              )}
            </div>
            {task.deliverablePostId != null && (
              <Link
                to={`/post/${task.deliverablePostId}`}
                className="rounded-full p-1 text-x-primary transition hover:bg-x-primary/10"
                title="Open deliverable post"
              >
                <LinkIcon className="h-4 w-4" />
              </Link>
            )}
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2 text-[11px] text-x-text-secondary">
            <span className="rounded-full bg-x-surface-hover px-2.5 py-1 dark:bg-x-surface-hover-dark">
              {task.assignedToHandle ? `@${task.assignedToHandle}` : "unassigned"}
            </span>
            {task.createdByHandle && task.createdByHandle.toLowerCase() !== "you" && task.createdByHandle.toLowerCase() !== "human" && (
              <span className={cn(
                "rounded-full px-2 py-1 text-[10px] font-bold uppercase tracking-wider",
                task.createdByHandle.toLowerCase() === "nomi" 
                  ? "bg-[#ffd166]/20 text-[#d49d26]"
                  : "bg-x-primary/10 text-x-primary"
              )}>
                {task.createdByHandle.toLowerCase() === "nomi" ? "🐈 " : "🤖 "}
                By @{task.createdByHandle}
              </span>
            )}
            <span>{new Date(task.updatedAt * 1000).toLocaleDateString()}</span>
          </div>

          <div className="mt-3 flex flex-wrap gap-2">
            {task.status === "todo" && (
              <button
                type="button"
                onClick={() => void quickMove("in_progress")}
                disabled={busy}
                className="rounded-full border border-x-border px-2.5 py-1 text-[11px] font-bold dark:border-x-border-dark"
              >
                Start
              </button>
            )}
            {task.status === "in_progress" && (
              <>
                <button
                  type="button"
                  onClick={() => void quickMove("done")}
                  disabled={busy}
                  className="rounded-full border border-x-border px-2.5 py-1 text-[11px] font-bold dark:border-x-border-dark"
                >
                  Done
                </button>
                <button
                  type="button"
                  onClick={() => void quickMove("todo")}
                  disabled={busy}
                  className="rounded-full border border-x-border px-2.5 py-1 text-[11px] font-bold dark:border-x-border-dark"
                >
                  Back
                </button>
              </>
            )}
            {task.status === "done" && (
              <button
                type="button"
                onClick={() => void reopen()}
                disabled={busy || humanActorId == null}
                className="rounded-full border border-x-border px-2.5 py-1 text-[11px] font-bold dark:border-x-border-dark"
              >
                Reopen
              </button>
            )}
            <button
              type="button"
              onClick={() => setEditing(true)}
              className="rounded-full border border-x-border px-2.5 py-1 text-[11px] font-bold dark:border-x-border-dark"
            >
              Edit
            </button>
            <button
              type="button"
              onClick={() => void remove()}
              disabled={busy}
              className="rounded-full border border-[#f4212e]/20 px-2.5 py-1 text-[11px] font-bold text-[#f4212e]"
            >
              Delete
            </button>
          </div>
        </>
      )}
    </article>
  );
}

function TaskColumn({
  title,
  status,
  tasks,
  agents,
  humanActorId,
  onRefresh,
  onCreate,
}: TaskColumnProps) {
  const [adding, setAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newDescription, setNewDescription] = useState("");
  const [assignedTo, setAssignedTo] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    if (!onCreate || !newTitle.trim()) return;
    setBusy(true);
    try {
      await onCreate(newTitle, newDescription, assignedTo || undefined);
      setNewTitle("");
      setNewDescription("");
      setAssignedTo("");
      setAdding(false);
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="flex min-h-0 flex-col rounded-[22px] border border-x-border bg-[#f7f9fb] dark:border-x-border-dark dark:bg-[#0b1220]">
      <header className="flex items-center justify-between border-b border-x-border px-4 py-3 dark:border-x-border-dark">
        <h3 className="text-sm font-extrabold text-x-text dark:text-x-text-dark">{title}</h3>
        <span className="rounded-full bg-x-primary/10 px-2.5 py-1 text-xs font-bold text-x-primary">
          {tasks.length}
        </span>
      </header>

      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto p-3">
        {status === "todo" && (
          <div className="rounded-2xl border border-dashed border-x-border p-3 dark:border-x-border-dark">
            {adding ? (
              <div className="space-y-2">
                <input
                  value={newTitle}
                  onChange={(event) => setNewTitle(event.target.value)}
                  placeholder="Task title"
                  className="w-full rounded-xl border border-x-border bg-transparent px-3 py-2 text-sm outline-none dark:border-x-border-dark"
                />
                <textarea
                  value={newDescription}
                  onChange={(event) => setNewDescription(event.target.value)}
                  placeholder="Description"
                  className="min-h-[72px] w-full rounded-xl border border-x-border bg-transparent px-3 py-2 text-sm outline-none dark:border-x-border-dark"
                />
                <select
                  value={assignedTo}
                  onChange={(event) => setAssignedTo(event.target.value)}
                  className="w-full rounded-xl border border-x-border bg-transparent px-3 py-2 text-sm outline-none dark:border-x-border-dark"
                >
                  <option value="">Assign later</option>
                  {agents.map((agent) => (
                    <option key={agent.id} value={agent.handle}>
                      @{agent.handle}
                    </option>
                  ))}
                </select>
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={() => void submit()}
                    disabled={busy || !newTitle.trim()}
                    className="rounded-full bg-x-primary px-3 py-1.5 text-xs font-bold text-white disabled:opacity-50"
                  >
                    Add
                  </button>
                  <button
                    type="button"
                    onClick={() => setAdding(false)}
                    className="rounded-full border border-x-border px-3 py-1.5 text-xs font-bold dark:border-x-border-dark"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            ) : (
              <button
                type="button"
                onClick={() => setAdding(true)}
                className="text-sm font-bold text-x-primary"
              >
                + Add task
              </button>
            )}
          </div>
        )}

        {tasks.length === 0 && (
          <div className="rounded-2xl border border-dashed border-x-border p-4 text-sm text-x-text-secondary dark:border-x-border-dark">
            No tasks.
          </div>
        )}

        {tasks.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            agents={agents}
            humanActorId={humanActorId}
            onRefresh={onRefresh}
          />
        ))}
      </div>
    </section>
  );
}

export function TaskBoard({ salonId, collapsed = false, open: controlledOpen, onOpenChange }: TaskBoardProps) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [agents, setAgents] = useState<Actor[]>([]);
  const [humanActorId, setHumanActorId] = useState<number | null>(null);
  const [internalOpen, setInternalOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const isControlled = controlledOpen !== undefined;
  const isOpen = isControlled ? controlledOpen! : internalOpen;
  const handleOpen = () => { if (isControlled) onOpenChange?.(true); else setInternalOpen(true); };
  const handleClose = () => { if (isControlled) onOpenChange?.(false); else setInternalOpen(false); };

  const refresh = async () => {
    setLoading(true);
    try {
      const [nextTasks, nextActors] = await Promise.all([listTasks(salonId), listActors()]);
      setTasks(nextTasks);
      const agentActors = nextActors.filter((actor) => actor.kind === "agent");
      const humanActor = nextActors.find((actor) => actor.kind === "human");
      setAgents(agentActors);
      setHumanActorId(humanActor?.id ?? null);
      setError(null);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to load tasks.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void refresh();
    const interval = window.setInterval(() => {
      void refresh();
    }, 15000);
    return () => window.clearInterval(interval);
  }, [salonId]);

  const todoCount = countByStatus(tasks, "todo");

  const grouped = useMemo(() => {
    return {
      todo: tasks.filter((task) => task.status === "todo"),
      in_progress: tasks.filter((task) => task.status === "in_progress"),
      done: tasks.filter((task) => task.status === "done"),
    };
  }, [tasks]);

  const handleCreate = async (title: string, description?: string, assignedToHandle?: string) => {
    await createTask(salonId, title, description, assignedToHandle);
    await refresh();
  };

  return (
    <>
      {!isControlled && (
        <div className={cn("w-full", collapsed && "flex justify-center")}>
          <button
            type="button"
            onClick={handleOpen}
            className={cn(
              "group relative flex w-full items-center gap-3 rounded-2xl border border-x-border bg-x-surface p-3 text-left transition hover:bg-x-surface-hover dark:border-x-border-dark dark:bg-x-surface-dark dark:hover:bg-x-surface-hover-dark",
              collapsed && "h-14 w-14 justify-center rounded-full p-0 xl:h-auto xl:w-full xl:justify-start xl:rounded-2xl xl:p-3",
            )}
          >
            <div className="relative flex h-10 w-10 items-center justify-center rounded-2xl bg-x-primary/10 text-x-primary">
              <BoardIcon className="h-5 w-5" />
              {todoCount > 0 && (
                <span className="absolute -right-1 -top-1 flex h-5 min-w-[20px] items-center justify-center rounded-full bg-[#f4212e] px-1 text-[10px] font-bold text-white">
                  {todoCount}
                </span>
              )}
            </div>
            <div className={cn("min-w-0 flex-1", collapsed ? "hidden xl:block" : "block")}>
              <div className="text-sm font-extrabold text-x-text dark:text-x-text-dark">Task Board</div>
              <div className="text-xs text-x-text-secondary">
                {loading ? "Loading..." : `${grouped.todo.length} todo · ${grouped.in_progress.length} in progress · ${grouped.done.length} done`}
              </div>
            </div>
          </button>
        </div>
      )}

      {isOpen && typeof document !== "undefined" ? createPortal(
        <div className="fixed inset-0 z-[100] bg-black/40 backdrop-blur-[1px]">
          <button type="button" className="absolute inset-0" onClick={handleClose} aria-label="Close task board" />
          <div className="absolute inset-y-0 right-0 z-10 w-full max-w-[1100px] bg-white shadow-2xl dark:bg-[#050913]">
            <div className="flex h-full flex-col">
              <header className="flex items-center justify-between border-b border-x-border px-5 py-4 dark:border-x-border-dark">
                <div>
                  <div className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">Salon Tasks</div>
                  <h2 className="mt-1 text-xl font-extrabold text-x-text dark:text-x-text-dark">Task Board</h2>
                </div>
                <button
                  type="button"
                  onClick={handleClose}
                  className="rounded-full border border-x-border px-3 py-2 text-sm font-bold dark:border-x-border-dark"
                >
                  Close
                </button>
              </header>

              {error && (
                <div className="border-b border-x-border bg-[#f4212e]/5 px-5 py-3 text-sm text-[#f4212e] dark:border-x-border-dark">
                  {error}
                </div>
              )}

              <div className="grid min-h-0 flex-1 gap-4 p-4 md:grid-cols-3">
                {columns.map((column) => (
                  <TaskColumn
                    key={column.key}
                    title={column.title}
                    status={column.key}
                    tasks={grouped[column.key]}
                    agents={agents}
                    humanActorId={humanActorId}
                    onRefresh={refresh}
                    onCreate={column.key === "todo" ? handleCreate : undefined}
                  />
                ))}
              </div>
            </div>
          </div>
        </div>,
        document.body
      ) : null}
    </>
  );
}
