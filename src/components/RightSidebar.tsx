import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import { cn } from "../lib/utils";
import { useLanguage } from "../lib/language";
import { useSalon } from "../lib/salon-context";
import {
  createTask,
  deleteSalon,
  deleteTask,
  downloadFileUrl,
  listTasks,
  searchFiles,
  updateTask,
  type FileInfo,
  type Task,
} from "../lib/client";
import { TaskBoard } from "./TaskBoard";

// ── file display helpers ──────────────────────────────────────────────────────

const KIND_LABELS: Record<string, string> = {
  pdf: "PDF",
  docx: "DOC",
  pptx: "PPT",
  xlsx: "XLS",
  csv: "CSV",
  image: "IMG",
  md: "MD",
};

const KIND_COLORS: Record<string, string> = {
  pdf: "bg-red-500/10 text-red-600",
  docx: "bg-blue-500/10 text-blue-600",
  pptx: "bg-orange-500/10 text-orange-600",
  xlsx: "bg-emerald-500/10 text-emerald-600",
  csv: "bg-teal-500/10 text-teal-600",
  image: "bg-x-primary/10 text-x-primary",
  md: "bg-zinc-500/10 text-zinc-600 dark:text-zinc-300",
};

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// ── sub-components ────────────────────────────────────────────────────────────

function CompactTaskRow({ task, onRefresh }: { task: Task; onRefresh: () => void }) {
  const [busy, setBusy] = useState(false);

  const move = async (status: Task["status"]) => {
    setBusy(true);
    try {
      await updateTask(task.id, { status });
      onRefresh();
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    setBusy(true);
    try {
      await deleteTask(task.id);
      onRefresh();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div
      className={cn(
        "group relative flex items-start gap-4 rounded-2xl px-4 py-3 transition-colors",
        task.status === "done" && "opacity-50",
        "hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark",
      )}
    >
      <div className="mt-1 shrink-0">
        {task.status === "todo" && <div className="h-4 w-4 rounded-full border-[2px] border-x-text-secondary/40 transition-colors group-hover:border-x-primary/50" />}
        {task.status === "in_progress" && <div className="h-4 w-4 rounded-full border-[2px] border-x-primary border-t-transparent animate-spin" />}
        {task.status === "done" && (
          <div className="flex h-4 w-4 items-center justify-center rounded-full bg-emerald-500 text-white">
            <svg viewBox="0 0 24 24" className="h-3 w-3" fill="none" stroke="currentColor" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
          </div>
        )}
      </div>
      <div className="min-w-0 flex-1">
        <div
          className={cn(
            "text-[15px] font-bold text-x-text dark:text-x-text-dark leading-tight transition-colors",
            task.status === "done" && "line-through font-normal text-x-text-secondary",
          )}
        >
          {task.title}
        </div>
        <div className="mt-1 flex flex-wrap items-center gap-1.5">
          {task.assignedToHandle && (
            <div className="text-[13px] text-x-text-secondary">@{task.assignedToHandle}</div>
          )}
          {task.createdByHandle && task.createdByHandle.toLowerCase() !== "you" && task.createdByHandle.toLowerCase() !== "human" && (
            <div className={cn(
              "rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider",
              task.createdByHandle.toLowerCase() === "nomi" 
                ? "bg-[#ffd166]/20 text-[#d49d26]"
                : "bg-x-primary/10 text-x-primary"
            )}>
              {task.createdByHandle.toLowerCase() === "nomi" ? "🐈 " : "🤖 "}
              By @{task.createdByHandle}
            </div>
          )}
        </div>

        <div className="mt-2 hidden flex-wrap items-center gap-3 group-hover:flex">
          {task.status === "todo" && (
            <button
              type="button"
              disabled={busy}
              onClick={(e) => { e.stopPropagation(); void move("in_progress"); }}
              className="text-[12px] font-bold text-x-primary hover:underline disabled:opacity-50"
            >
              Start
            </button>
          )}
          {task.status === "in_progress" && (
            <>
              <button
                type="button"
                disabled={busy}
                onClick={(e) => { e.stopPropagation(); void move("done"); }}
                className="text-[12px] font-bold text-emerald-600 hover:underline disabled:opacity-50"
              >
                Done
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={(e) => { e.stopPropagation(); void move("todo"); }}
                className="text-[12px] font-bold text-x-text-secondary hover:underline disabled:opacity-50"
              >
                Back
              </button>
            </>
          )}
          {task.status === "done" && (
            <button
              type="button"
              disabled={busy}
              onClick={(e) => { e.stopPropagation(); void move("todo"); }}
              className="text-[12px] font-bold text-x-text-secondary hover:underline disabled:opacity-50"
            >
              Reopen
            </button>
          )}
          <button
            type="button"
            disabled={busy}
            onClick={(e) => { e.stopPropagation(); void remove(); }}
            className="ml-auto text-[12px] font-bold text-red-500 hover:underline disabled:opacity-50"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}

function SidebarFileRow({ file, snippet }: { file: FileInfo; snippet?: string }) {
  return (
    <a
      href={downloadFileUrl(file.id)}
      download={file.originalName}
      className="group relative flex items-center gap-4 rounded-2xl px-4 py-3 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
    >
      <div
        className={cn(
          "flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-[10px] font-extrabold transition-colors",
          KIND_COLORS[file.kind] ?? "bg-x-border text-x-text dark:bg-x-border-dark dark:text-x-text-dark",
        )}
      >
        {KIND_LABELS[file.kind] ?? "FILE"}
      </div>
      <div className="min-w-0 flex-1">
        <div className="truncate text-[15px] font-bold text-x-text dark:text-x-text-dark">
          {file.originalName}
        </div>
        {snippet ? (
          <div className="mt-0.5 line-clamp-2 text-[13px] text-x-text-secondary">{snippet}</div>
        ) : (
          <div className="mt-0.5 text-[13px] text-x-text-secondary">{formatSize(file.sizeBytes)}</div>
        )}
      </div>
    </a>
  );
}

// ── main component ────────────────────────────────────────────────────────────

const DONE_PREVIEW = 3;
const GENERAL_SALON_ID = 1;

export function RightSidebar() {
  const navigate = useNavigate();
  const { language, t } = useLanguage();
  const { activeSalonId, salons, setActiveSalonId, refreshSalons } = useSalon();
  const salonId = activeSalonId ?? 1;

  // tasks
  const [tasks, setTasks] = useState<Task[]>([]);
  const [taskBoardOpen, setTaskBoardOpen] = useState(false);
  const [showAddTask, setShowAddTask] = useState(false);
  const [newTaskTitle, setNewTaskTitle] = useState("");
  const [showAllDone, setShowAllDone] = useState(false);

  // files
  const [fileResults, setFileResults] = useState<{ file: FileInfo; snippet: string }[]>([]);
  const [fileQuery, setFileQuery] = useState("");
  const [fileLoading, setFileLoading] = useState(true);

  useEffect(() => {
    let alive = true;
    const load = async () => {
      try {
        const data = await listTasks(salonId);
        if (alive) setTasks(data);
      } catch {
        // silently ignore
      }
    };
    void load();
    const interval = setInterval(() => void load(), 15_000);
    return () => {
      alive = false;
      clearInterval(interval);
    };
  }, [salonId]);

  useEffect(() => {
    let alive = true;
    const load = async () => {
      setFileLoading(true);
      try {
        const results = await searchFiles(salonId, fileQuery || undefined, 20);
        if (alive) setFileResults(results);
      } catch {
        // silently ignore
      } finally {
        if (alive) setFileLoading(false);
      }
    };
    const timer = setTimeout(() => void load(), fileQuery ? 300 : 0);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [salonId, fileQuery]);

  const refreshTasks = () => {
    listTasks(salonId)
      .then(setTasks)
      .catch(() => {});
  };

  const handleQuickAdd = async () => {
    const title = newTaskTitle.trim();
    if (!title) return;
    await createTask(salonId, title);
    setNewTaskTitle("");
    setShowAddTask(false);
    refreshTasks();
  };

  const todoTasks = tasks.filter((t) => t.status === "todo");
  const inProgressTasks = tasks.filter((t) => t.status === "in_progress");
  const doneTasks = tasks.filter((t) => t.status === "done");
  const visibleDone = showAllDone ? doneTasks : doneTasks.slice(0, DONE_PREVIEW);
  const hiddenDoneCount = doneTasks.length - DONE_PREVIEW;

  const handleDeleteSalon = async (salonIdToDelete: number, _salonName: string) => {
    if (activeSalonId === salonIdToDelete) {
      setActiveSalonId(GENERAL_SALON_ID);
      navigate("/");
    }

    try {
      await deleteSalon(salonIdToDelete);
    } catch (err) {
      console.error("deleteSalon failed:", err);
    } finally {
      await refreshSalons();
    }
  };

  return (
    <aside className="sticky top-0 hidden h-screen w-[350px] flex-col overflow-y-auto px-3 py-4 lg:flex">

      {/* ── 顶部搜索/入口区 ─────────────────── */}
      <div className="mb-4 flex items-center justify-between px-4 py-2">
        <div className="flex items-center gap-3 w-full rounded-full bg-x-surface-hover dark:bg-x-surface-hover-dark px-4 py-3 focus-within:ring-1 focus-within:ring-x-primary transition-shadow">
          <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text-secondary" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="7" />
            <line x1="21" y1="21" x2="16" y2="16" />
          </svg>
          <input
            value={fileQuery}
            onChange={(e) => setFileQuery(e.target.value)}
            className="w-full border-none bg-transparent text-[15px] outline-none placeholder:text-x-text-secondary text-x-text dark:text-x-text-dark"
            placeholder="Search files..."
          />
        </div>
      </div>

      <section className="mb-6 w-full">
        <div className="mb-2 px-4">
          <span className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
            {t("nav.workspace")}
          </span>
        </div>
        <div className="flex flex-col gap-0.5">
          {salons.map((salon) => {
            const active = salon.id === activeSalonId;
            return (
              <div
                key={salon.id}
                className={cn(
                  "group relative flex items-center gap-3 rounded-2xl px-2 py-1 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark",
                  active && "bg-x-primary/8 text-x-primary dark:bg-x-primary/10",
                )}
              >
                <button
                  type="button"
                  onClick={() => {
                    setActiveSalonId(salon.id);
                    navigate("/");
                  }}
                  className="flex min-w-0 flex-1 items-center gap-4 rounded-xl px-2 py-2 text-left"
                >
                  <div
                    className={cn(
                      "flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-sm font-bold transition-colors",
                      active
                        ? "bg-x-primary text-white"
                        : "bg-x-border text-x-text dark:bg-x-border-dark dark:text-x-text-dark group-hover:bg-x-border/80 dark:group-hover:bg-x-border-dark/80",
                    )}
                  >
                    {salon.name.charAt(0).toUpperCase()}
                  </div>
                  <div className="min-w-0 flex-1">
                    <div
                      className={cn(
                        "truncate text-[15px] font-bold leading-tight",
                        active ? "text-x-primary" : "text-x-text dark:text-x-text-dark",
                      )}
                    >
                      {salon.name}
                    </div>
                    {salon.topic && (
                      <div className="mt-0.5 truncate text-[13px] text-x-text-secondary">
                        {salon.topic}
                      </div>
                    )}
                  </div>
                </button>
                {salon.id !== GENERAL_SALON_ID && (
                  <button
                    type="button"
                    aria-label={language === "zh" ? `删除${salon.name}` : `Delete ${salon.name}`}
                    title={language === "zh" ? "删除工作区" : "Delete workspace"}
                    onClick={(event) => {
                      event.stopPropagation();
                      void handleDeleteSalon(salon.id, salon.name);
                    }}
                    className="mr-1 flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-x-text-secondary transition-colors hover:bg-red-500/10 hover:text-red-500"
                  >
                    <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
                      <path d="M9 3h6l1 2h4v2H4V5h4l1-2zm1 6h2v8h-2V9zm4 0h2v8h-2V9zM7 9h2v8H7V9zm-1 12a2 2 0 0 1-2-2V8h16v11a2 2 0 0 1-2 2H6z" />
                    </svg>
                  </button>
                )}
              </div>
            );
          })}
        </div>
      </section>

      {/* ── 任务板 section ──────────────────────────────────── */}
      {fileQuery === "" && (
        <section className="w-full mb-6">
          <div className="mb-2 flex items-center justify-between px-4">
            <span className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
              Tasks
            </span>
            <button
              type="button"
              onClick={() => setTaskBoardOpen(true)}
              className="text-[13px] font-bold text-x-primary hover:underline"
            >
              Full Board →
            </button>
          </div>

          <div className="flex flex-col gap-0.5">
            {inProgressTasks.map((task) => (
              <CompactTaskRow key={task.id} task={task} onRefresh={refreshTasks} />
            ))}
            {todoTasks.map((task) => (
              <CompactTaskRow key={task.id} task={task} onRefresh={refreshTasks} />
            ))}
            {visibleDone.map((task) => (
              <CompactTaskRow key={task.id} task={task} onRefresh={refreshTasks} />
            ))}

            {!showAllDone && hiddenDoneCount > 0 && (
              <button
                type="button"
                onClick={() => setShowAllDone(true)}
                className="mt-2 px-4 text-left text-[13px] font-bold text-x-primary hover:underline"
              >
                Show {hiddenDoneCount} more done...
              </button>
            )}
            {showAllDone && doneTasks.length > DONE_PREVIEW && (
              <button
                type="button"
                onClick={() => setShowAllDone(false)}
                className="mt-2 px-4 text-left text-[13px] font-bold text-x-primary hover:underline"
              >
                Show less
              </button>
            )}

            {tasks.length === 0 && (
              <p className="px-4 py-2 text-[15px] text-x-text-secondary">No tasks yet.</p>
            )}

            {/* quick add */}
            <div className="mt-2 px-4">
              {showAddTask ? (
                <div className="flex gap-2">
                  <input
                    autoFocus
                    value={newTaskTitle}
                    onChange={(e) => setNewTaskTitle(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") void handleQuickAdd();
                      if (e.key === "Escape") setShowAddTask(false);
                    }}
                    placeholder="Task title..."
                    className="min-w-0 flex-1 rounded-full border border-x-border bg-transparent px-3 py-1.5 text-[15px] outline-none focus:border-x-primary dark:border-x-border-dark dark:text-x-text-dark"
                  />
                  <button
                    type="button"
                    onClick={() => void handleQuickAdd()}
                    className="rounded-full bg-x-primary px-3 py-1.5 text-sm font-bold text-white hover:bg-x-primary-hover transition-colors"
                  >
                    Add
                  </button>
                  <button
                    type="button"
                    onClick={() => { setShowAddTask(false); setNewTaskTitle(""); }}
                    className="rounded-full border border-x-border px-2.5 py-1.5 text-sm dark:border-x-border-dark text-x-text-secondary hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark transition-colors"
                  >
                    ✕
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() => setShowAddTask(true)}
                  className="text-[15px] font-bold text-x-primary hover:underline"
                >
                  + Add task
                </button>
              )}
            </div>
          </div>
        </section>
      )}

      {/* ── 项目库 section ─────────────────────────────────── */}
      <section className="w-full">
        <div className="mb-2 px-4">
          <span className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
            Files
          </span>
        </div>

        {fileLoading && fileResults.length === 0 ? (
          <p className="px-4 py-2 text-[15px] text-x-text-secondary">Loading...</p>
        ) : fileResults.length === 0 ? (
          <p className="px-4 py-2 text-[15px] text-x-text-secondary">
            {fileQuery ? "No matching files." : "No files uploaded yet."}
          </p>
        ) : (
          <div className="flex flex-col gap-0.5">
            {fileResults.map((r) => (
              <SidebarFileRow
                key={r.file.id}
                file={r.file}
                snippet={fileQuery ? r.snippet || undefined : undefined}
              />
            ))}
          </div>
        )}
      </section>

      {/* ── footer ──────── */}
      <footer className="mt-auto px-4 pt-6 pb-2 text-[13px] text-x-text-secondary">
        © 2026 超级智能办公室
      </footer>

      {/* 全板 slide-over */}
      <TaskBoard salonId={salonId} open={taskBoardOpen} onOpenChange={setTaskBoardOpen} />
    </aside>
  );
}
