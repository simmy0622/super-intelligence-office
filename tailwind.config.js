/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // X.com 风格配色
        x: {
          // 主色调 - X Blue
          primary: "rgb(29, 155, 240)",
          "primary-hover": "rgb(26, 140, 216)",
          
          // 背景色
          background: "rgb(255, 255, 255)",
          "background-dark": "rgb(0, 0, 0)",
          
          // 卡片/表面
          surface: "rgb(255, 255, 255)",
          "surface-dark": "rgb(22, 24, 28)",
          "surface-hover": "rgba(0, 0, 0, 0.03)",
          "surface-hover-dark": "rgba(255, 255, 255, 0.03)",
          
          // 边框
          border: "rgb(239, 243, 244)",
          "border-dark": "rgb(47, 51, 54)",
          
          // 文字
          text: "rgb(15, 20, 25)",
          "text-dark": "rgb(231, 233, 234)",
          "text-secondary": "rgb(83, 100, 113)",
          "text-secondary-dark": "rgb(113, 118, 123)",
          
          // 交互色
          like: "rgb(249, 24, 128)",
          "like-hover": "rgba(249, 24, 128, 0.1)",
          repost: "rgb(0, 186, 124)",
          "repost-hover": "rgba(0, 186, 124, 0.1)",
          reply: "rgb(29, 155, 240)",
          "reply-hover": "rgba(29, 155, 240, 0.1)",
          share: "rgb(29, 155, 240)",
          "share-hover": "rgba(29, 155, 240, 0.1)",
        },
        
        // 保留原有 salon 配色用于兼容
        salon: {
          ink: "#171c21",
          muted: "#3f4851",
          line: "#bfc7d3",
          panel: "#f0f4fb",
          shell: "#f7f9ff",
          shellStrong: "#e4e8f0",
          accent: "#00629d",
          accentBright: "#1d9bf0",
          accentSoft: "#cfe5ff",
          warm: "#db7e00",
          success: "#0f8f63",
          danger: "#ba1a1a",
        },
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "Helvetica",
          "Arial",
          "sans-serif",
        ],
      },
      boxShadow: {
        panel: "0 16px 30px rgba(23, 28, 33, 0.06)",
        fab: "0 4px 12px rgba(29, 155, 240, 0.4)",
        modal: "0 25px 50px -12px rgba(0, 0, 0, 0.25)",
      },
      borderRadius: {
        xxl: "1.5rem",
      },
      animation: {
        "like-bounce": "likeBounce 0.45s cubic-bezier(0.175, 0.885, 0.32, 1.275)",
        "fade-in": "fadeIn 0.2s ease-out",
        "slide-up": "slideUp 0.3s ease-out",
        "scale-in": "scaleIn 0.2s ease-out",
      },
      keyframes: {
        likeBounce: {
          "0%": { transform: "scale(1)" },
          "50%": { transform: "scale(1.3)" },
          "100%": { transform: "scale(1)" },
        },
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { transform: "translateY(10px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        scaleIn: {
          "0%": { transform: "scale(0.95)", opacity: "0" },
          "100%": { transform: "scale(1)", opacity: "1" },
        },
      },
    },
  },
  plugins: [],
};
