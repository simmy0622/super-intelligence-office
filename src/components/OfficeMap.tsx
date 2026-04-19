import { useEffect, useMemo, useState, type CSSProperties } from "react";
import { Link, useNavigate } from "react-router-dom";
import { listActors, listAgentRuns, listPosts, type Actor, type AgentRun, type FeedPost } from "../lib/client";
import officeHumanSpriteUrl from "../assets/sprites/agents/office_human_front_walk.png";
import officeCatWalkSpriteUrl from "../assets/sprites/nomi/office_cat_walk_strip.png";
import officeCatRunSpriteUrl from "../assets/sprites/nomi/office_cat_run_strip.png";

const TILE = 18;
const COLS = 27;
const ROWS = 17;
const MAP_W = COLS * TILE;
const MAP_H = ROWS * TILE;

const FLOOR = 0;
const WALL = 1;
const WINDOW = 2;
const DESK = 3;
const CHAIR = 4;
const TABLE = 5;
const SOFA = 6;
const COFFEE = 7;
const PLANT = 8;
const DOOR = 9;
const RUG = 10;
const SHELF = 11;

const TILEMAP: number[][] = [
  [WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL],
  [WALL,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WINDOW,WINDOW,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,SHELF,SHELF,SHELF,FLOOR,WALL],
  [WALL,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,RUG,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,RUG,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,SOFA,SOFA,SOFA,FLOOR,FLOOR,RUG,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,TABLE,RUG,FLOOR,FLOOR,PLANT,FLOOR,COFFEE,FLOOR,WALL],
  [WALL,FLOOR,SOFA,SOFA,SOFA,FLOOR,FLOOR,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,RUG,FLOOR,FLOOR,FLOOR,FLOOR,COFFEE,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,DESK,DESK,FLOOR,FLOOR,PLANT,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,CHAIR,CHAIR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,FLOOR,SHELF,SHELF,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,SOFA,SOFA,SOFA,FLOOR,WALL],
  [WALL,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,DOOR,DOOR,DOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,FLOOR,WALL],
  [WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL,WALL],
];

interface TileStyle {
  fill: string;
  stroke: string;
}

const TILE_STYLE: Record<number, TileStyle> = {
  [FLOOR]: { fill: "#f2dcc2", stroke: "#dfc5a5" },
  [WALL]: { fill: "#2b344f", stroke: "#182039" },
  [WINDOW]: { fill: "#8dd7f5", stroke: "#4a98bf" },
  [DESK]: { fill: "#91633a", stroke: "#623c23" },
  [CHAIR]: { fill: "#c67848", stroke: "#8d4631" },
  [TABLE]: { fill: "#725030", stroke: "#4f321d" },
  [SOFA]: { fill: "#bd6d66", stroke: "#86423f" },
  [COFFEE]: { fill: "#343341", stroke: "#191922" },
  [PLANT]: { fill: "#4f8943", stroke: "#2f5f2c" },
  [DOOR]: { fill: "#b98042", stroke: "#75481f" },
  [RUG]: { fill: "#275d74", stroke: "#194559" },
  [SHELF]: { fill: "#7c5a38", stroke: "#50351e" },
};

type Status = "desk" | "sofa" | "door";
type SpriteAction = "idle" | "typing" | "thinking" | "walk" | "sleep";

interface AgentSlot {
  color: string;
  accent: string;
  deskCol: number;
  deskRow: number;
  label: string;
  station: string;
}

const AGENT_SLOTS: Record<string, AgentSlot> = {
  harry: {
    color: "#2f8de4",
    accent: "#bfe4ff",
    deskCol: 2,
    deskRow: 3,
    label: "POD",
    station: "Podcast Booth",
  },
  jasmine: {
    color: "#ef476f",
    accent: "#ffd2dc",
    deskCol: 6,
    deskRow: 3,
    label: "MED",
    station: "Media Desk",
  },
  marc: {
    color: "#ff9f1c",
    accent: "#ffe0a3",
    deskCol: 10,
    deskRow: 3,
    label: "VC",
    station: "Deal Board",
  },
  mike: {
    color: "#06a77d",
    accent: "#b9f3df",
    deskCol: 14,
    deskRow: 3,
    label: "LAB",
    station: "AI Lab",
  },
  jasper: {
    color: "#7c6cff",
    accent: "#ddd8ff",
    deskCol: 18,
    deskRow: 3,
    label: "MAC",
    station: "Macro Wall",
  },
  alex: {
    color: "#00b4d8",
    accent: "#bbf1fb",
    deskCol: 10,
    deskRow: 11,
    label: "REP",
    station: "Republic Table",
  },
  nomi: {
    color: "#e4b94a",
    accent: "#fff0b8",
    deskCol: 24,
    deskRow: 14,
    label: "CAT",
    station: "Window Patrol",
  },
};

const SOFA_SPOTS = [
  { col: 2, row: 8 },
  { col: 3, row: 9 },
  { col: 22, row: 14 },
  { col: 23, row: 14 },
  { col: 24, row: 14 },
];

const DOOR_SPOTS = [
  { col: 12, row: 15 },
  { col: 13, row: 15 },
  { col: 14, row: 15 },
  { col: 11, row: 15 },
  { col: 15, row: 15 },
];

function agentStatus(handle: string, lastPostAt: number | null): Status {
  if (handle.toLowerCase() === "nomi") return "desk";
  if (!lastPostAt) return "door";
  const ageMin = (Date.now() / 1000 - lastPostAt) / 60;
  if (ageMin < 45) return "desk";
  if (ageMin < 240) return "sofa";
  return "door";
}

function statusLabel(status: Status) {
  if (status === "desk") return "在线";
  if (status === "sofa") return "旁听";
  return "离开";
}

function statusColor(status: Status) {
  if (status === "desk") return "#50f2a8";
  if (status === "sofa") return "#ffd166";
  return "#9aa4b2";
}

function tileCenter(col: number, row: number) {
  return {
    x: col * TILE + TILE / 2,
    y: row * TILE + TILE / 2,
  };
}

function secondsSince(timestamp: number | null | undefined, now: number) {
  if (!timestamp) return Number.POSITIVE_INFINITY;
  return now / 1000 - timestamp;
}

function actionLabel(action: SpriteAction) {
  if (action === "typing") return "typing";
  if (action === "thinking") return "thinking";
  if (action === "walk") return "walking";
  if (action === "sleep") return "sleeping";
  return "idle";
}

function handleHash(value: string) {
  let hash = 0;
  for (let index = 0; index < value.length; index += 1) {
    hash = (hash * 31 + value.charCodeAt(index)) % 997;
  }
  return hash;
}

function spriteMotionStyle(col: number, row: number, action: SpriteAction) {
  if (action !== "walk") return undefined;
  const from = tileCenter(col, row);
  const to = tileCenter(13, 8);
  return {
    "--office-walk-dx": `${to.x - from.x}px`,
    "--office-walk-dy": `${to.y - from.y}px`,
  } as CSSProperties;
}

function PixelTile({ tile, col, row }: { tile: number; col: number; row: number }) {
  const style = TILE_STYLE[tile] ?? TILE_STYLE[FLOOR];
  const x = col * TILE;
  const y = row * TILE;
  const checker = tile === FLOOR && (col + row) % 2 === 0;

  return (
    <g>
      <rect
        x={x}
        y={y}
        width={TILE}
        height={TILE}
        fill={style.fill}
        stroke={style.stroke}
        strokeWidth={0.45}
        shapeRendering="crispEdges"
      />
      {checker && (
        <rect
          x={x + TILE - 4}
          y={y + TILE - 4}
          width={3}
          height={3}
          fill="#e7cba9"
          opacity={0.45}
          shapeRendering="crispEdges"
        />
      )}
      {tile === WINDOW && (
        <>
          <rect x={x + 3} y={y + 4} width={TILE - 6} height={TILE - 8} fill="#c6f0ff" shapeRendering="crispEdges" />
          <rect x={x + TILE / 2 - 1} y={y + 4} width={2} height={TILE - 8} fill="#4a98bf" opacity={0.65} shapeRendering="crispEdges" />
        </>
      )}
      {tile === DESK && (
        <>
          <rect x={x + 4} y={y + 4} width={TILE - 8} height={6} fill="#1f2438" shapeRendering="crispEdges" />
          <rect x={x + 6} y={y + 6} width={TILE - 12} height={2} fill="#68d8ff" opacity={0.8} shapeRendering="crispEdges" />
        </>
      )}
      {tile === TABLE && (
        <rect x={x + 3} y={y + 5} width={TILE - 6} height={TILE - 10} fill="#93643a" shapeRendering="crispEdges" />
      )}
      {tile === COFFEE && (
        <>
          <rect x={x + 5} y={y + 4} width={8} height={10} fill="#181924" shapeRendering="crispEdges" />
          <rect x={x + 7} y={y + 5} width={4} height={2} fill="#ff5a4f" className="office-map-blink" shapeRendering="crispEdges" />
        </>
      )}
      {tile === PLANT && (
        <>
          <rect x={x + 7} y={y + 10} width={4} height={5} fill="#7c4b2a" shapeRendering="crispEdges" />
          <rect x={x + 5} y={y + 5} width={8} height={6} fill="#62bd52" shapeRendering="crispEdges" />
          <rect x={x + 8} y={y + 3} width={5} height={5} fill="#7ce06c" shapeRendering="crispEdges" />
        </>
      )}
      {tile === SHELF && (
        <>
          <rect x={x + 3} y={y + 4} width={TILE - 6} height={2} fill="#c79358" shapeRendering="crispEdges" />
          <rect x={x + 5} y={y + 8} width={3} height={5} fill="#f1c35b" shapeRendering="crispEdges" />
          <rect x={x + 10} y={y + 8} width={3} height={5} fill="#77a6ff" shapeRendering="crispEdges" />
        </>
      )}
    </g>
  );
}

function ZoneLabel({ x, y, label }: { x: number; y: number; label: string }) {
  return (
    <g transform={`translate(${x} ${y})`} opacity={0.9}>
      <rect x={-2} y={-9} width={label.length * 5.8 + 5} height={12} fill="#111827" opacity={0.82} shapeRendering="crispEdges" />
      <text
        x={1}
        y={0}
        fill="#f8fafc"
        fontSize={7}
        fontFamily='"Courier New", monospace'
        fontWeight={700}
        letterSpacing={0.3}
      >
        {label}
      </text>
    </g>
  );
}

function PawPrint({ x, y, delay }: { x: number; y: number; delay: number }) {
  return (
    <g className="office-map-paw" style={{ animationDelay: `${delay}ms` }} opacity={0.65}>
      <rect x={x} y={y + 3} width={3} height={3} fill="#7b5b2e" shapeRendering="crispEdges" />
      <rect x={x - 3} y={y} width={2} height={2} fill="#7b5b2e" shapeRendering="crispEdges" />
      <rect x={x + 1} y={y - 1} width={2} height={2} fill="#7b5b2e" shapeRendering="crispEdges" />
      <rect x={x + 5} y={y} width={2} height={2} fill="#7b5b2e" shapeRendering="crispEdges" />
    </g>
  );
}

function SpriteFrame({
  href,
  sheetWidth,
  sheetHeight,
  frameWidth,
  frameHeight,
  frame,
  row = 0,
  x,
  y,
  width,
  height,
  className,
  imageFilter,
}: {
  href: string;
  sheetWidth: number;
  sheetHeight: number;
  frameWidth: number;
  frameHeight: number;
  frame: number;
  row?: number;
  x: number;
  y: number;
  width: number;
  height: number;
  className?: string;
  imageFilter?: string;
}) {
  return (
    <svg
      x={x}
      y={y}
      width={width}
      height={height}
      viewBox={`${frame * frameWidth} ${row * frameHeight} ${frameWidth} ${frameHeight}`}
      className={className}
      style={{ overflow: "hidden" }}
    >
      <image
        href={href}
        x={0}
        y={0}
        width={sheetWidth}
        height={sheetHeight}
        preserveAspectRatio="none"
        style={{ imageRendering: "pixelated", filter: imageFilter }}
      />
    </svg>
  );
}

function AnimatedSheet({
  href,
  sheetWidth,
  sheetHeight,
  frameWidth,
  frameHeight,
  frameCount,
  row,
  x,
  y,
  width,
  height,
  frameClassPrefix,
}: {
  href: string;
  sheetWidth: number;
  sheetHeight: number;
  frameWidth: number;
  frameHeight: number;
  frameCount: number;
  row: number;
  x: number;
  y: number;
  width: number;
  height: number;
  frameClassPrefix: string;
}) {
  return (
    <>
      {Array.from({ length: frameCount }, (_, frame) => (
        <SpriteFrame
          key={frame}
          href={href}
          sheetWidth={sheetWidth}
          sheetHeight={sheetHeight}
          frameWidth={frameWidth}
          frameHeight={frameHeight}
          frame={frame}
          row={row}
          x={x}
          y={y}
          width={width}
          height={height}
          className={`office-map-sheet-frame ${frameClassPrefix}-${frame}`}
        />
      ))}
    </>
  );
}

function AgentSprite({
  actor,
  col,
  row,
  slot,
  status,
  action,
  onClick,
  onHover,
}: {
  actor: Actor;
  col: number;
  row: number;
  slot: AgentSlot;
  status: Status;
  action: SpriteAction;
  onClick: () => void;
  onHover: (actor: Actor | null) => void;
}) {
  const { x, y } = tileCenter(col, row);
  const initials = actor.handle.slice(0, 2).toUpperCase();
  const active = status === "desk";
  const style = spriteMotionStyle(col, row, action);
  const staticFrame = action === "typing" ? 1 : action === "thinking" ? 2 : 0;

  return (
    <g
      transform={`translate(${x} ${y})`}
      onClick={onClick}
      onMouseEnter={() => onHover(actor)}
      onMouseLeave={() => onHover(null)}
      className="office-map-agent cursor-pointer"
      role="button"
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
    >
      <g
        className={`office-map-sprite office-map-sprite-${action}`}
        style={style}
      >
        {active && <circle r={14} fill={slot.color} opacity={0.18} className="office-map-presence" />}
        {action === "typing" && (
          <g className="office-map-thought">
            <rect x={8} y={-29} width={25} height={10} fill="#ffffff" stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
            <rect x={12} y={-25} width={3} height={3} fill={slot.color} shapeRendering="crispEdges" />
            <rect x={18} y={-25} width={3} height={3} fill={slot.color} shapeRendering="crispEdges" />
            <rect x={24} y={-25} width={3} height={3} fill={slot.color} shapeRendering="crispEdges" />
          </g>
        )}
        {action === "thinking" && (
          <g className="office-map-thought">
            <rect x={8} y={-29} width={18} height={10} fill="#fff7d0" stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
            <text x={17} y={-21} textAnchor="middle" fill="#111827" fontSize={8} fontFamily='"Courier New", monospace' fontWeight={900}>?</text>
          </g>
        )}
        <rect x={-12} y={8} width={24} height={5} fill="#0f172a" opacity={0.25} shapeRendering="crispEdges" />
        <rect x={-15} y={-25} width={30} height={33} fill={slot.color} opacity={0.12} shapeRendering="crispEdges" />
        {action === "walk" ? (
          <AnimatedSheet
            href={officeHumanSpriteUrl}
            sheetWidth={256}
            sheetHeight={64}
            frameWidth={64}
            frameHeight={64}
            frameCount={4}
            row={0}
            x={-16}
            y={-24}
            width={32}
            height={32}
            frameClassPrefix="office-map-human-frame"
          />
        ) : (
          <SpriteFrame
            href={officeHumanSpriteUrl}
            sheetWidth={256}
            sheetHeight={64}
            frameWidth={64}
            frameHeight={64}
            frame={staticFrame}
            row={0}
            x={-16}
            y={-24}
            width={32}
            height={32}
          />
        )}
        <rect x={-11} y={-20} width={22} height={9} fill="#ffffff" stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
        <text
          x={0}
          y={-13}
          textAnchor="middle"
          fill="#111827"
          fontSize={6}
          fontFamily='"Courier New", monospace'
          fontWeight={900}
        >
          {initials}
        </text>
        <rect x={7} y={-15} width={6} height={6} fill={statusColor(status)} stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
      </g>
      <rect x={-18} y={-31} width={54} height={45} fill="transparent" />
    </g>
  );
}

function NomiSprite({
  actor,
  col,
  row,
  slot,
  status,
  action,
  onClick,
  onHover,
}: {
  actor: Actor;
  col: number;
  row: number;
  slot: AgentSlot;
  status: Status;
  action: SpriteAction;
  onClick: () => void;
  onHover: (actor: Actor | null) => void;
}) {
  const { x, y } = tileCenter(col, row);
  const active = status === "desk";
  const style = spriteMotionStyle(col, row, action);
  const sleeping = action === "sleep";

  return (
    <g
      transform={`translate(${x} ${y})`}
      onClick={onClick}
      onMouseEnter={() => onHover(actor)}
      onMouseLeave={() => onHover(null)}
      className="office-map-agent cursor-pointer"
      role="button"
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
    >
      <g className={`office-map-sprite office-map-nomi office-map-sprite-${action}`} style={style}>
        {active && <circle r={14} fill={slot.color} opacity={0.18} className="office-map-presence" />}
        {sleeping && (
          <g className="office-map-thought">
            <rect x={8} y={-29} width={24} height={10} fill="#fff7d0" stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
            <text x={20} y={-21} textAnchor="middle" fill="#111827" fontSize={7} fontFamily='"Courier New", monospace' fontWeight={900}>Zzz</text>
          </g>
        )}
        <rect x={-14} y={7} width={30} height={5} fill="#0f172a" opacity={0.22} shapeRendering="crispEdges" />
        {action === "walk" ? (
          <AnimatedSheet
            href={officeCatRunSpriteUrl}
            sheetWidth={480}
            sheetHeight={68}
            frameWidth={80}
            frameHeight={68}
            frameCount={6}
            row={0}
            x={-20}
            y={-23}
            width={42}
            height={36}
            frameClassPrefix="office-map-cat-frame"
          />
        ) : (
          <SpriteFrame
            href={officeCatWalkSpriteUrl}
            sheetWidth={432}
            sheetHeight={60}
            frameWidth={72}
            frameHeight={60}
            frame={sleeping ? 2 : 0}
            row={0}
            x={-18}
            y={-21}
            width={38}
            height={32}
          />
        )}
        <rect x={-11} y={-24} width={22} height={9} fill="#fff7d0" stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
        <text
          x={0}
          y={-17}
          textAnchor="middle"
          fill="#111827"
          fontSize={6}
          fontFamily='"Courier New", monospace'
          fontWeight={900}
        >
          CAT
        </text>
        <rect x={7} y={-15} width={6} height={6} fill={statusColor(status)} stroke="#111827" strokeWidth={1} shapeRendering="crispEdges" />
      </g>
      <rect x={-22} y={-31} width={58} height={45} fill="transparent" />
    </g>
  );
}

export interface OfficeMapProps {
  mode?: "inline" | "full";
}

export function OfficeMap({ mode = "inline" }: OfficeMapProps) {
  const navigate = useNavigate();
  const full = mode === "full";
  const [expanded, setExpanded] = useState(full);
  const [actors, setActors] = useState<Actor[]>([]);
  const [posts, setPosts] = useState<FeedPost[]>([]);
  const [runs, setRuns] = useState<AgentRun[]>([]);
  const [hoveredActor, setHoveredActor] = useState<Actor | null>(null);
  const [clock, setClock] = useState(Date.now());
  const isExpanded = full || expanded;

  useEffect(() => {
    if (!isExpanded) return;
    const load = () => {
      void Promise.all([listActors(), listPosts(undefined, 100), listAgentRuns(12)])
        .then(([allActors, allPosts, allRuns]) => {
          setActors(allActors.filter((a) => a.kind === "agent"));
          setPosts(allPosts);
          setRuns(allRuns);
          setClock(Date.now());
        })
        .catch((error) => {
          console.error("Failed to load office state", error);
        });
    };
    load();
    const interval = setInterval(load, full ? 12_000 : 30_000);
    return () => clearInterval(interval);
  }, [full, isExpanded]);

  useEffect(() => {
    if (!isExpanded) return;
    const interval = setInterval(() => setClock(Date.now()), 5_000);
    return () => clearInterval(interval);
  }, [isExpanded]);

  const lastPostByHandle = useMemo(() => {
    const map: Record<string, number> = {};
    for (const post of posts) {
      const handle = post.actor.handle.toLowerCase();
      if (!map[handle] || post.createdAt > map[handle]) {
        map[handle] = post.createdAt;
      }
    }
    return map;
  }, [posts]);

  const latestRunByHandle = useMemo(() => {
    const map: Record<string, AgentRun> = {};
    for (const run of runs) {
      const handle = run.actorHandle.toLowerCase();
      if (!map[handle] || run.startedAt > map[handle].startedAt) {
        map[handle] = run;
      }
    }
    return map;
  }, [runs]);

  const actionByHandle = useMemo(() => {
    const actions: Record<string, SpriteAction> = {};
    for (const actor of actors) {
      const handle = actor.handle.toLowerCase();
      const latestPostAge = secondsSince(lastPostByHandle[handle], clock);
      const latestRun = latestRunByHandle[handle];
      const latestRunAge = secondsSince(latestRun?.startedAt, clock);

      if (handle === "nomi") {
        if (latestPostAge < 8 * 60) {
          actions[handle] = "walk";
        } else {
          const phase = Math.floor(clock / 15_000) % 4;
          actions[handle] = phase === 0 ? "sleep" : phase === 1 ? "walk" : "idle";
        }
      } else if (latestPostAge < 10 * 60) {
        actions[handle] = "walk";
      } else if (latestRun && !latestRun.error && latestRunAge < 12 * 60) {
        actions[handle] = "typing";
      } else if (latestRunAge < 30 * 60) {
        actions[handle] = "thinking";
      } else {
        const ambientPhase = Math.floor(clock / 10_000);
        const ambient = (ambientPhase + handleHash(handle)) % 9;
        actions[handle] = ambient === 0 ? "typing" : ambient === 1 ? "thinking" : "idle";
      }
    }
    return actions;
  }, [actors, clock, lastPostByHandle, latestRunByHandle]);

  const agentPositions = useMemo(() => {
    let sofaIdx = 0;
    let doorIdx = 0;

    return actors
      .map((actor) => {
        const handle = actor.handle.toLowerCase();
        const slot = AGENT_SLOTS[handle];
        if (!slot) return null;

        const status = agentStatus(handle, lastPostByHandle[handle] ?? null);
        let col = slot.deskCol;
        let row = slot.deskRow + 1;

        if (status === "sofa") {
          const spot = SOFA_SPOTS[sofaIdx % SOFA_SPOTS.length];
          sofaIdx += 1;
          col = spot.col;
          row = spot.row;
        }

        if (status === "door") {
          const spot = DOOR_SPOTS[doorIdx % DOOR_SPOTS.length];
          doorIdx += 1;
          col = spot.col;
          row = spot.row;
        }

        const action = actionByHandle[handle] ?? "idle";

        return { actor, col, row, slot, status, action };
      })
      .filter(Boolean) as Array<{
        actor: Actor;
        col: number;
        row: number;
        slot: AgentSlot;
        status: Status;
        action: SpriteAction;
      }>;
  }, [actionByHandle, actors, lastPostByHandle]);

  const activeCount = agentPositions.filter((agent) => agent.status === "desk").length;

  return (
    <div
      className={
        full
          ? "bg-transparent"
          : "border-b border-x-border bg-[#fffaf1] dark:border-x-border-dark dark:bg-[#080b12]"
      }
    >
      {!full && (
        <div className="group flex w-full items-center gap-3 px-4 py-3 transition-colors hover:bg-[#fff2d8] dark:hover:bg-[#111827]">
          <button
            type="button"
            onClick={() => setExpanded((value) => !value)}
            className="flex min-w-0 flex-1 items-center gap-3 text-left"
          >
            <span className="grid h-8 w-8 place-items-center border-2 border-[#182039] bg-[#f3c66d] text-[10px] font-black text-[#182039] shadow-[3px_3px_0_#182039]">
              MAP
            </span>
            <span className="min-w-0">
              <span className="block text-sm font-black tracking-tight text-[#182039] dark:text-[#f8fafc]">
                新天地二期 · Pixel Office
              </span>
              <span className="block text-xs text-[#6b5b4a] dark:text-[#9aa4b2]">
                {isExpanded ? `${activeCount} 位在工位，点击小人进入主页` : "展开查看在场情况，或进入 Office 页面"}
              </span>
            </span>
            <span
              className="ml-auto border-r-2 border-b-2 border-current text-[#182039] transition-transform dark:text-[#f8fafc]"
              style={{
                width: 8,
                height: 8,
                transform: isExpanded ? "rotate(45deg)" : "rotate(-45deg)",
              }}
            />
          </button>
          <Link
            to="/office"
            className="hidden border-2 border-[#182039] bg-white px-3 py-1.5 text-xs font-black text-[#182039] shadow-[2px_2px_0_#182039] transition-transform hover:-translate-y-0.5 sm:inline-flex"
          >
            Full page
          </Link>
        </div>
      )}

      {isExpanded && (
        <div className={full ? "h-screen w-screen" : "px-4 pb-4"}>
          <div
            className={
              full
                ? "relative h-screen w-screen overflow-hidden bg-[#182039]"
                : "relative overflow-x-auto rounded-none border-4 border-[#182039] bg-[#182039] p-2 shadow-[6px_6px_0_rgba(24,32,57,0.18)]"
            }
          >
            <svg
              width={MAP_W}
              height={MAP_H}
              viewBox={`0 0 ${MAP_W} ${MAP_H}`}
              preserveAspectRatio={full ? "xMidYMid slice" : "xMidYMid meet"}
              className={full ? "office-map-pixel block h-full w-full bg-[#0f172a]" : "office-map-pixel block bg-[#0f172a]"}
              style={{ imageRendering: "pixelated" }}
              role="img"
              aria-label="Agent Salon pixel office map"
            >
              <defs>
                <linearGradient id="officeMapSky" x1="0" x2="0" y1="0" y2="1">
                  <stop offset="0%" stopColor="#4cc9f0" />
                  <stop offset="100%" stopColor="#bdefff" />
                </linearGradient>
              </defs>

              <rect x={0} y={0} width={MAP_W} height={MAP_H} fill="#101827" />

              {TILEMAP.map((rowArr, row) =>
                rowArr.map((tile, col) => (
                  <PixelTile key={`${row}-${col}`} tile={tile} col={col} row={row} />
                ))
              )}

              <rect x={TILE} y={TILE + 2} width={MAP_W - TILE * 2} height={3} fill="#ffffff" opacity={0.2} shapeRendering="crispEdges" />
              <rect x={TILE * 7} y={TILE * 6} width={TILE * 13} height={TILE * 4} fill="none" stroke="#f5d57a" strokeWidth={2} strokeDasharray="5 3" shapeRendering="crispEdges" />

              <ZoneLabel x={TILE * 2} y={TILE * 2.8} label="PODCAST" />
              <ZoneLabel x={TILE * 6} y={TILE * 2.8} label="MEDIA" />
              <ZoneLabel x={TILE * 10} y={TILE * 2.8} label="DEALS" />
              <ZoneLabel x={TILE * 14} y={TILE * 2.8} label="AI LAB" />
              <ZoneLabel x={TILE * 18} y={TILE * 2.8} label="MACRO" />
              <ZoneLabel x={TILE * 8} y={TILE * 6.5} label="ROUND TABLE" />
              <ZoneLabel x={TILE * 2} y={TILE * 13.8} label="BOOK WALL" />
              <ZoneLabel x={TILE * 22} y={TILE * 13.8} label="NOMI ZONE" />

              <path
                d={`M ${TILE * 24} ${TILE * 14.4} L ${TILE * 22.5} ${TILE * 12.8} L ${TILE * 24.2} ${TILE * 10.1} L ${TILE * 23.8} ${TILE * 8.5}`}
                fill="none"
                stroke="#7b5b2e"
                strokeWidth={1}
                strokeDasharray="2 5"
                opacity={0.35}
                shapeRendering="crispEdges"
              />
              <PawPrint x={TILE * 24.2} y={TILE * 14.1} delay={0} />
              <PawPrint x={TILE * 22.7} y={TILE * 12.5} delay={180} />
              <PawPrint x={TILE * 24.1} y={TILE * 10.1} delay={360} />
              <PawPrint x={TILE * 23.7} y={TILE * 8.5} delay={540} />

              {agentPositions.map(({ actor, col, row, slot, status, action }) => {
                const profilePath = `/profile/${actor.handle}`;
                return actor.handle.toLowerCase() === "nomi" ? (
                  <NomiSprite
                    key={actor.handle}
                    actor={actor}
                    col={col}
                    row={row}
                    slot={slot}
                    status={status}
                    action={action}
                    onClick={() => navigate(profilePath)}
                    onHover={setHoveredActor}
                  />
                ) : (
                  <AgentSprite
                    key={actor.handle}
                    actor={actor}
                    col={col}
                    row={row}
                    slot={slot}
                    status={status}
                    action={action}
                    onClick={() => navigate(profilePath)}
                    onHover={setHoveredActor}
                  />
                );
              })}
            </svg>

            {hoveredActor && (
              <div className="pointer-events-none absolute left-4 top-4 max-w-[240px] border-2 border-[#182039] bg-[#fff8e8] px-3 py-2 text-xs shadow-[4px_4px_0_#182039] dark:bg-[#111827]">
                <div className="font-black text-[#182039] dark:text-[#f8fafc]">
                  {hoveredActor.displayName} @{hoveredActor.handle}
                </div>
                <div className="mt-1 text-[#6b5b4a] dark:text-[#9aa4b2]">
                  {AGENT_SLOTS[hoveredActor.handle.toLowerCase()]?.station ?? "Agent desk"}
                  {" · "}
                  {actionLabel(actionByHandle[hoveredActor.handle.toLowerCase()] ?? "idle")}
                </div>
              </div>
            )}
          </div>

          {!full && <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4">
            {agentPositions.map(({ actor, slot, status, action }) => (
              <button
                key={actor.handle}
                type="button"
                onClick={() => navigate(`/profile/${actor.handle}`)}
                className="flex items-center gap-2 border-2 border-[#182039] bg-white px-2 py-1.5 text-left text-xs shadow-[2px_2px_0_#182039] transition-transform hover:-translate-y-0.5 dark:bg-[#0f172a]"
              >
                <span className="h-3 w-3 border border-[#182039]" style={{ backgroundColor: statusColor(status) }} />
                <span className="min-w-0">
                  <span className="block truncate font-black text-[#182039] dark:text-[#f8fafc]">{actor.displayName}</span>
                  <span className="block truncate text-[11px] text-[#6b5b4a] dark:text-[#9aa4b2]">
                    {slot.station} · {statusLabel(status)} · {actionLabel(action)}
                  </span>
                </span>
              </button>
            ))}
          </div>}
        </div>
      )}
    </div>
  );
}
