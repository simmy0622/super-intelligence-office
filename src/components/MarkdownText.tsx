import { Fragment, type ReactNode } from "react";
import { cn } from "../lib/utils";

interface MarkdownTextProps {
  content: string;
  className?: string;
}

type MarkdownBlock =
  | { type: "heading"; level: number; content: string }
  | { type: "paragraph"; content: string }
  | { type: "blockquote"; content: string }
  | { type: "unordered-list"; items: string[] }
  | { type: "ordered-list"; items: string[] }
  | { type: "code"; content: string };

const INLINE_TOKEN_PATTERN =
  /(`[^`]+`|\[([^\]]+)\]\(([^)\s]+)\)|\*\*([^*]+)\*\*|__([^_]+)__|~~([^~]+)~~|\*([^*\n]+)\*|_([^_\n]+)_)/g;

function isBlockBoundary(line: string) {
  return (
    line.trim() === "" ||
    /^```/.test(line) ||
    /^(#{1,6})\s+/.test(line) ||
    /^\s*>\s?/.test(line) ||
    /^\s*[-*+]\s+/.test(line) ||
    /^\s*\d+\.\s+/.test(line)
  );
}

function parseBlocks(markdown: string): MarkdownBlock[] {
  const lines = markdown.replace(/\r\n/g, "\n").trim().split("\n");
  const blocks: MarkdownBlock[] = [];

  for (let index = 0; index < lines.length; ) {
    const line = lines[index];

    if (line.trim() === "") {
      index += 1;
      continue;
    }

    if (/^```/.test(line)) {
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !/^```/.test(lines[index])) {
        codeLines.push(lines[index]);
        index += 1;
      }
      if (index < lines.length && /^```/.test(lines[index])) {
        index += 1;
      }
      blocks.push({ type: "code", content: codeLines.join("\n") });
      continue;
    }

    const headingMatch = line.match(/^(#{1,6})\s+(.*)$/);
    if (headingMatch) {
      blocks.push({
        type: "heading",
        level: headingMatch[1].length,
        content: headingMatch[2],
      });
      index += 1;
      continue;
    }

    if (/^\s*>\s?/.test(line)) {
      const quoteLines: string[] = [];
      while (index < lines.length && /^\s*>\s?/.test(lines[index])) {
        quoteLines.push(lines[index].replace(/^\s*>\s?/, ""));
        index += 1;
      }
      blocks.push({ type: "blockquote", content: quoteLines.join("\n") });
      continue;
    }

    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*[-*+]\s+/.test(lines[index])) {
        items.push(lines[index].replace(/^\s*[-*+]\s+/, ""));
        index += 1;
      }
      blocks.push({ type: "unordered-list", items });
      continue;
    }

    if (/^\s*\d+\.\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*\d+\.\s+/.test(lines[index])) {
        items.push(lines[index].replace(/^\s*\d+\.\s+/, ""));
        index += 1;
      }
      blocks.push({ type: "ordered-list", items });
      continue;
    }

    const paragraphLines = [line];
    index += 1;
    while (index < lines.length && !isBlockBoundary(lines[index])) {
      paragraphLines.push(lines[index]);
      index += 1;
    }
    blocks.push({ type: "paragraph", content: paragraphLines.join("\n") });
  }

  return blocks;
}

function renderTextWithLineBreaks(text: string, keyPrefix: string) {
  return text.split("\n").map((segment, index) => (
    <Fragment key={`${keyPrefix}-line-${index}`}>
      {index > 0 && <br />}
      {segment}
    </Fragment>
  ));
}

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  let lastIndex = 0;

  for (const match of text.matchAll(INLINE_TOKEN_PATTERN)) {
    const [fullMatch] = match;
    const matchIndex = match.index ?? 0;

    if (matchIndex > lastIndex) {
      nodes.push(
        <Fragment key={`${keyPrefix}-text-${lastIndex}`}>
          {renderTextWithLineBreaks(text.slice(lastIndex, matchIndex), `${keyPrefix}-${lastIndex}`)}
        </Fragment>
      );
    }

    if (fullMatch.startsWith("`")) {
      nodes.push(
        <code
          key={`${keyPrefix}-code-${matchIndex}`}
          className="rounded bg-x-surface-hover px-1 py-0.5 font-mono text-[0.95em] dark:bg-x-surface-hover-dark"
        >
          {fullMatch.slice(1, -1)}
        </code>
      );
    } else if (match[2] && match[3]) {
      const href = match[3];
      const safeHref = /^(https?:\/\/|mailto:)/i.test(href) ? href : "#";
      nodes.push(
        <a
          key={`${keyPrefix}-link-${matchIndex}`}
          href={safeHref}
          target="_blank"
          rel="noreferrer"
          className="pointer-events-auto text-x-primary underline underline-offset-2 hover:opacity-80"
          onClick={(event) => event.stopPropagation()}
        >
          {renderInline(match[2], `${keyPrefix}-link-label-${matchIndex}`)}
        </a>
      );
    } else if (match[4] || match[5]) {
      nodes.push(
        <strong key={`${keyPrefix}-strong-${matchIndex}`} className="font-bold">
          {renderInline(match[4] ?? match[5] ?? "", `${keyPrefix}-strong-child-${matchIndex}`)}
        </strong>
      );
    } else if (match[6]) {
      nodes.push(
        <span key={`${keyPrefix}-strike-${matchIndex}`} className="line-through opacity-80">
          {renderInline(match[6], `${keyPrefix}-strike-child-${matchIndex}`)}
        </span>
      );
    } else if (match[7] || match[8]) {
      nodes.push(
        <em key={`${keyPrefix}-em-${matchIndex}`} className="italic">
          {renderInline(match[7] ?? match[8] ?? "", `${keyPrefix}-em-child-${matchIndex}`)}
        </em>
      );
    }

    lastIndex = matchIndex + fullMatch.length;
  }

  if (lastIndex < text.length) {
    nodes.push(
      <Fragment key={`${keyPrefix}-tail-${lastIndex}`}>
        {renderTextWithLineBreaks(text.slice(lastIndex), `${keyPrefix}-tail-${lastIndex}`)}
      </Fragment>
    );
  }

  return nodes;
}

export function markdownToPlainText(markdown: string): string {
  return markdown
    .replace(/\r\n/g, "\n")
    .replace(/```[\w-]*\n([\s\S]*?)```/g, "$1")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/^\s*>\s?/gm, "")
    .replace(/^\s*[-*+]\s+/gm, "")
    .replace(/^\s*\d+\.\s+/gm, "")
    .replace(/(\*\*|__|\*|_|~~)/g, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

export function MarkdownText({ content, className }: MarkdownTextProps) {
  const blocks = parseBlocks(content);

  return (
    <div className={cn("space-y-3 break-words text-x-text dark:text-x-text-dark", className)}>
      {blocks.map((block, index) => {
        if (block.type === "heading") {
          const headingClassName =
            block.level === 1
              ? "text-xl font-extrabold"
              : block.level === 2
                ? "text-lg font-bold"
                : "text-base font-bold";
          return (
            <div key={`heading-${index}`} className={headingClassName}>
              {renderInline(block.content, `heading-${index}`)}
            </div>
          );
        }

        if (block.type === "blockquote") {
          return (
            <blockquote
              key={`quote-${index}`}
              className="border-l-4 border-x-primary/40 pl-4 text-x-text-secondary dark:text-x-text-secondary"
            >
              {renderInline(block.content, `quote-${index}`)}
            </blockquote>
          );
        }

        if (block.type === "unordered-list") {
          return (
            <ul key={`ul-${index}`} className="list-disc space-y-1 pl-5">
              {block.items.map((item, itemIndex) => (
                <li key={`ul-${index}-${itemIndex}`}>{renderInline(item, `ul-${index}-${itemIndex}`)}</li>
              ))}
            </ul>
          );
        }

        if (block.type === "ordered-list") {
          return (
            <ol key={`ol-${index}`} className="list-decimal space-y-1 pl-5">
              {block.items.map((item, itemIndex) => (
                <li key={`ol-${index}-${itemIndex}`}>{renderInline(item, `ol-${index}-${itemIndex}`)}</li>
              ))}
            </ol>
          );
        }

        if (block.type === "code") {
          return (
            <pre
              key={`code-${index}`}
              className="overflow-x-auto rounded-2xl bg-x-surface-hover px-4 py-3 font-mono text-[13px] leading-6 dark:bg-x-surface-hover-dark"
            >
              <code>{block.content}</code>
            </pre>
          );
        }

        return (
          <p key={`paragraph-${index}`} className="whitespace-normal leading-normal">
            {renderInline(block.content, `paragraph-${index}`)}
          </p>
        );
      })}
    </div>
  );
}
