import { memo, useEffect, useState, type MouseEvent } from "react";
import { cn } from "../lib/utils";
import { motion, AnimatePresence } from "framer-motion";

interface LikeButtonProps {
  liked: boolean;
  count: number;
  onLike: () => void;
}

const NUM_GROUPS = 7;
const colors = [
  "#f91880", // Pink
  "#ff7a00", // Orange
  "#7856ff", // Purple
  "#fda4c8", // Light Pink
  "#1d9bf0", // Blue
  "#00ba7c", // Green
  "#f91880", // Pink again
];

// 预计算粒子轨道和物理属性，生成包含主粒子、副粒子和小闪光星的三层爆发效果
const particles = Array.from({ length: NUM_GROUPS }).flatMap((_, index) => {
  const angle = (index * 360) / NUM_GROUPS;
  const rad = (angle * Math.PI) / 180;
  
  // 1. 主粒子
  const distance1 = 20;
  const p1 = {
    x: Math.cos(rad) * distance1,
    y: Math.sin(rad) * distance1,
    color: colors[index % colors.length],
    size: 3.5,
    delay: 0,
    gravity: 8 // 微弱重力
  };

  // 2. 副粒子
  const rad2 = ((angle + 25) * Math.PI) / 180;
  const distance2 = 28;
  const p2 = {
    x: Math.cos(rad2) * distance2,
    y: Math.sin(rad2) * distance2,
    color: colors[(index + 1) % colors.length],
    size: 1.5,
    delay: 0.02,
    gravity: 12
  };

  return [p1, p2];
});

export const LikeButton = memo(function LikeButton({ liked: initialLiked, count: initialCount, onLike }: LikeButtonProps) {
  const [isLiked, setIsLiked] = useState(initialLiked);
  const [count, setCount] = useState(initialCount);
  const [burstKey, setBurstKey] = useState(0);

  useEffect(() => {
    setIsLiked(initialLiked);
    setCount(initialCount);
  }, [initialLiked, initialCount]);

  const handleLike = (e: MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const newLiked = !isLiked;
    setIsLiked(newLiked);
    setCount((prev) => Math.max(0, prev + (newLiked ? 1 : -1)));
    
    if (newLiked) {
      setBurstKey((prev) => prev + 1);
    }

    onLike();
  };

  return (
    <button
      type="button"
      onClick={handleLike}
      aria-pressed={isLiked}
      className={cn(
        "group relative flex w-[72px] shrink-0 items-center gap-1.5 text-sm leading-none transition-colors",
        isLiked ? "text-[#f91880]" : "text-x-text-secondary hover:text-[#f91880]"
      )}
    >
      <div className="relative flex h-9 w-9 shrink-0 items-center justify-center rounded-full transition-colors group-hover:bg-[#f91880]/10">
        
        {/* Expanding Burst Ring */}
        <AnimatePresence>
          {isLiked && (
            <motion.div
              key={`ring-${burstKey}`}
              className="absolute inset-0 rounded-full border-[#f91880]"
              initial={{ scale: 0.1, opacity: 1, borderWidth: "16px" }}
              animate={{ scale: 1.8, opacity: 0, borderWidth: "0px" }}
              transition={{ duration: 0.5, ease: "easeOut" }}
            />
          )}
        </AnimatePresence>

        {/* Physics Particles */}
        <AnimatePresence>
          {isLiked && particles.map((p, i) => (
            <motion.div
              key={`particle-${burstKey}-${i}`}
              className="absolute rounded-full pointer-events-none"
              style={{
                backgroundColor: p.color,
                width: p.size,
                height: p.size,
                left: "50%",
                top: "50%",
                marginLeft: -p.size / 2,
                marginTop: -p.size / 2,
              }}
              initial={{ x: 0, y: 0, scale: 0, opacity: 1 }}
              animate={{
                x: [0, p.x, p.x * 1.05], 
                y: [0, p.y, p.y + p.gravity],
                scale: [0, 1, 1, 0],
                opacity: [1, 1, 0.6, 0]
              }}
              transition={{
                duration: 0.55, // 回归 x.com 的清脆迅速
                delay: p.delay,
                x: { 
                  times: [0, 0.5, 1], 
                  ease: ["easeOut", "linear"] 
                },
                y: { 
                  times: [0, 0.5, 1], 
                  ease: ["easeOut", "easeIn"] 
                },
                scale: { times: [0, 0.2, 0.6, 1] },
                opacity: { times: [0, 0.2, 0.6, 1] }
              }}
            />
          ))}
        </AnimatePresence>

        {/* Heart SVG */}
        <span className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
          <motion.svg
            viewBox="0 0 24 24"
            className="block h-5 w-5 shrink-0"
            initial={false}
            animate={{
              scale: isLiked ? [1, 1.45, 0.85, 1.15, 1] : 1,
            }}
            transition={{
              duration: 0.55,
              times: [0, 0.25, 0.45, 0.7, 1],
              ease: "easeInOut"
            }}
            style={{
              transformBox: "fill-box",
              transformOrigin: "center",
              willChange: "transform",
              transform: "translateZ(0)",
            }}
          >
            {/* Outlined Heart (Unliked) */}
            <path
              d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
              fill="none"
              stroke="currentColor"
              strokeWidth={1.5}
              opacity={isLiked ? 0 : 1}
              style={{ transition: "opacity 0.2s" }}
            />
            {/* Filled Heart (Liked) */}
            <path
              d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
              fill="currentColor"
              stroke="none"
              opacity={isLiked ? 1 : 0}
              style={{ transition: "opacity 0.2s" }}
            />
          </motion.svg>
        </span>
      </div>

      {/* Animated Counter */}
      <div className="relative flex h-5 w-[3ch] items-center overflow-hidden">
        <AnimatePresence mode="popLayout" initial={false}>
          <motion.span
            key={count}
            initial={{ y: isLiked ? 15 : -15, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: isLiked ? -15 : 15, opacity: 0 }}
            transition={{ duration: 0.3, type: "spring", bounce: 0.3 }}
            className="block w-full tabular-nums"
          >
            {count > 0 ? count : ""}
          </motion.span>
        </AnimatePresence>
      </div>
    </button>
  );
});
