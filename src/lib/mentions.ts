export interface MentionMatch {
  start: number;
  end: number;
  query: string;
}

export function findMentionAtCursor(value: string, cursor: number): MentionMatch | null {
  const prefix = value.slice(0, cursor);
  const atIndex = prefix.lastIndexOf("@");

  if (atIndex === -1) return null;
  if (atIndex > 0 && !/\s/.test(value[atIndex - 1])) return null;

  const query = value.slice(atIndex + 1, cursor);
  if (/\s/.test(query)) return null;

  return {
    start: atIndex,
    end: cursor,
    query,
  };
}

export function insertMention(value: string, match: MentionMatch, handle: string) {
  const mention = `@${handle} `;
  const nextValue = `${value.slice(0, match.start)}${mention}${value.slice(match.end)}`;
  const cursor = match.start + mention.length;
  return { nextValue, cursor };
}
