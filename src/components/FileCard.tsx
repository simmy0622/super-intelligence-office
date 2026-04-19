import { downloadFileUrl, type FileInfo } from "../lib/client";
import { cn } from "../lib/utils";

interface FileCardProps {
  file: FileInfo;
  compact?: boolean;
}

const KIND_LABELS: Record<string, string> = {
  pdf: "PDF",
  docx: "DOCX",
  pptx: "PPTX",
  xlsx: "XLSX",
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

export function FileCard({ file, compact = false }: FileCardProps) {
  const url = downloadFileUrl(file.id);

  if (file.kind === "image") {
    return (
      <a
        href={url}
        download={file.originalName}
        onClick={(event) => event.stopPropagation()}
        className="pointer-events-auto mt-3 block overflow-hidden rounded-2xl border border-x-border bg-x-surface transition-colors hover:bg-x-surface-hover dark:border-x-border-dark dark:bg-x-surface-dark dark:hover:bg-x-surface-hover-dark"
      >
        <img
          src={url}
          alt={file.originalName}
          loading="lazy"
          className={cn("w-full object-cover", compact ? "h-28" : "max-h-[420px]")}
        />
        <FileMeta file={file} />
      </a>
    );
  }

  return (
    <a
      href={url}
      download={file.originalName}
      onClick={(event) => event.stopPropagation()}
      className="pointer-events-auto mt-3 flex items-center gap-3 rounded-2xl border border-x-border bg-x-surface px-3 py-3 transition-colors hover:bg-x-surface-hover dark:border-x-border-dark dark:bg-x-surface-dark dark:hover:bg-x-surface-hover-dark"
    >
      <div
        className={cn(
          "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-xs font-black",
          KIND_COLORS[file.kind] ?? "bg-x-surface-hover text-x-text-secondary"
        )}
      >
        {KIND_LABELS[file.kind] ?? "FILE"}
      </div>
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-bold text-x-text dark:text-x-text-dark">
          {file.originalName}
        </div>
        <div className="mt-0.5 text-xs text-x-text-secondary">
          {KIND_LABELS[file.kind] ?? file.kind.toUpperCase()} · {formatSize(file.sizeBytes)}
        </div>
      </div>
      <span className="rounded-full bg-x-primary/10 px-3 py-1 text-xs font-bold text-x-primary">
        Download
      </span>
    </a>
  );
}

function FileMeta({ file }: { file: FileInfo }) {
  return (
    <div className="flex items-center justify-between gap-3 border-t border-x-border px-3 py-2 text-xs dark:border-x-border-dark">
      <span className="truncate font-semibold text-x-text dark:text-x-text-dark">
        {file.originalName}
      </span>
      <span className="shrink-0 text-x-text-secondary">{formatSize(file.sizeBytes)}</span>
    </div>
  );
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
