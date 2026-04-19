const AGENT_AVATAR_MAP: Record<string, string> = {
  jasmine: "/agent-avatars/agent-4.jpg",
  marc: "/agent-avatars/agent-2.jpg",
  harry: "/agent-avatars/harry.png",
  mike: "/agent-avatars/agent-1.jpg",
  jasper: "/agent-avatars/agent-5.jpg",
  alex: "/agent-avatars/agent-6.jpg",
  nomi: "/agent-avatars/nuomi.jpg",
  nuomi: "/agent-avatars/nuomi.jpg",
};

function normalizeAvatarKey(value: string) {
  return value?.trim?.()?.toLowerCase?.() || "";
}

function isDirectImageSource(value: string) {
  return (
    value.startsWith("data:") ||
    value.startsWith("http://") ||
    value.startsWith("https://") ||
    value.startsWith("/")
  );
}

export function avatarSource(seed: string, label: string) {
  const safeSeed = seed || label || "User";
  if (isDirectImageSource(safeSeed)) {
    return safeSeed;
  }
  const mappedAvatar = AGENT_AVATAR_MAP[normalizeAvatarKey(safeSeed)];
  if (mappedAvatar) {
    return mappedAvatar;
  }

  return avatarDataUrl(safeSeed, label || "User");
}

function paletteFromSeed(seed: string) {
  const safeSeed = seed || "User";
  let hash = 0;
  for (const char of safeSeed) {
    hash = (hash * 31 + char.charCodeAt(0)) % 360;
  }

  const hue = Math.abs(hash);
  return {
    base: `hsl(${hue} 72% 58%)`,
    soft: `hsl(${(hue + 36) % 360} 80% 87%)`,
    deep: `hsl(${(hue + 220) % 360} 52% 22%)`,
  };
}

export function avatarDataUrl(seed: string, label: string) {
  const safeLabel = label || "User";
  const { base, soft, deep } = paletteFromSeed(seed);
  const initials = safeLabel
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("")
    .slice(0, 2);

  const svg = `
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 96 96" role="img" aria-label="${safeLabel}">
      <defs>
        <linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="${soft}" />
          <stop offset="100%" stop-color="${base}" />
        </linearGradient>
      </defs>
      <rect width="96" height="96" rx="26" fill="url(#g)" />
      <circle cx="70" cy="26" r="14" fill="rgba(255,255,255,0.28)" />
      <path d="M18 73c8-14 18-21 30-21s22 7 30 21" fill="rgba(255,255,255,0.18)" />
      <text
        x="48"
        y="56"
        text-anchor="middle"
        font-family="Inter, Arial, sans-serif"
        font-size="28"
        font-weight="700"
        fill="${deep}"
      >${initials || "AI"}</text>
    </svg>
  `.trim();

  return `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(svg)}`;
}
