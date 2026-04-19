import { motion } from "framer-motion";
import { cn } from "../lib/utils";

interface AnimatedButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  type?: "button" | "submit" | "reset";
}

const variants = {
  primary: "bg-x-primary text-white hover:bg-x-primary-hover",
  secondary: "bg-transparent border border-x-border hover:bg-x-surface-hover",
  ghost: "bg-transparent hover:bg-x-surface-hover",
  danger: "bg-transparent text-red-500 hover:bg-red-500/10",
};

const sizes = {
  sm: "px-4 py-1.5 text-sm",
  md: "px-5 py-2 text-base",
  lg: "px-6 py-3 text-lg font-bold",
};

export function AnimatedButton({
  children,
  onClick,
  disabled,
  className,
  variant = "primary",
  size = "md",
  type = "button",
}: AnimatedButtonProps) {
  return (
    <motion.button
      type={type}
      onClick={onClick}
      disabled={disabled}
      whileHover={{ scale: disabled ? 1 : 1.02 }}
      whileTap={{ scale: disabled ? 1 : 0.98 }}
      transition={{ type: "spring", stiffness: 400, damping: 17 }}
      className={cn(
        "rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
        variants[variant],
        sizes[size],
        className
      )}
    >
      {children}
    </motion.button>
  );
}
