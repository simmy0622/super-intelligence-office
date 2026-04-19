import { cn } from "../lib/utils";
import { avatarSource } from "../lib/avatar";

interface AvatarProps {
  seed?: string | null;
  label: string;
  size?: "xs" | "sm" | "md" | "lg" | "xl";
  className?: string;
}

const sizeMap = {
  xs: "h-6 w-6",
  sm: "h-10 w-10",
  md: "h-12 w-12",
  lg: "h-20 w-20",
  xl: "h-28 w-28",
};

export function Avatar({ seed, label, size = "md", className }: AvatarProps) {
  const source = avatarSource(seed ?? label, label);

  return (
    <img
      src={source}
      alt={label}
      className={cn(
        "aspect-square shrink-0 rounded-full border-2 border-x-background object-cover dark:border-x-background-dark bg-x-surface-hover dark:bg-x-surface-hover-dark",
        sizeMap[size],
        className
      )}
    />
  );
}
