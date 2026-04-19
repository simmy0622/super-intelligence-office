import { Link, Outlet, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { LeftNav } from "./LeftNav";
import { RightSidebar } from "./RightSidebar";
import { useEffect, useRef, useState } from "react";
import { getApiKeyStatus } from "../lib/client";
import { useLanguage } from "../lib/language";


// 悬浮发帖按钮
function FloatingActionButton({ onClick }: { onClick: () => void }) {
  return (
    <motion.button
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
      onClick={onClick}
      className="fixed bottom-6 right-6 z-50 flex h-14 w-14 items-center justify-center rounded-full bg-x-primary text-white shadow-fab transition-colors hover:bg-x-primary-hover md:hidden"
    >
      <svg viewBox="0 0 24 24" className="h-6 w-6" fill="currentColor">
        <path d="M23 3c-6.62-.1-10.38 2.421-13.05 6.03C7.29 12.61 6 17.331 6 22h2c0-1.007.07-2.012.19-3H12c4.1 0 7.48-3.082 7.94-7.054C22.79 10.147 23.17 6.359 23 3zm-7 8h-1.5v2H16c.63-.016 1.2-.08 1.72-.188C16.95 15.24 14.68 17 12 17H8.55c.57-2.512 1.57-4.851 3-6.78 2.16-2.912 5.29-4.911 9.45-5.187C20.95 8.079 19.9 11 16 11zM4 9V6H1V4h3V1h2v3h3v2H6v3H4z" />
      </svg>
    </motion.button>
  );
}

// 滚动进度条
function ScrollProgress({ scrollContainerRef }: { scrollContainerRef: { current: HTMLElement | null } }) {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      const container = scrollContainerRef.current;
      const scrollTop = container?.scrollTop ?? window.scrollY;
      const docHeight = container
        ? container.scrollHeight - container.clientHeight
        : document.documentElement.scrollHeight - window.innerHeight;
      const scrollPercent = docHeight > 0 ? (scrollTop / docHeight) * 100 : 0;
      setProgress(scrollPercent);
    };

    const container = scrollContainerRef.current;
    handleScroll();
    container?.addEventListener("scroll", handleScroll, { passive: true });
    window.addEventListener("resize", handleScroll);
    return () => {
      container?.removeEventListener("scroll", handleScroll);
      window.removeEventListener("resize", handleScroll);
    };
  }, [scrollContainerRef]);

  return (
    <div className="fixed left-0 top-0 z-[100] h-[2px] w-full bg-transparent">
      <motion.div
        className="h-full bg-x-primary"
        style={{ width: `${progress}%` }}
        transition={{ duration: 0.1 }}
      />
    </div>
  );
}

function NoKeyBanner() {
  const [show, setShow] = useState(false);
  const { t } = useLanguage();
  const settingsLabel = t("app.settingsLink");
  const [beforeLink, afterLink] = t("app.deepseekMissing", { settings: settingsLabel }).split(settingsLabel);

  useEffect(() => {
    void getApiKeyStatus("deepseek").then((s) => setShow(!s.configured));
  }, []);

  if (!show) return null;
  return (
    <div className="fixed top-0 left-0 right-0 z-[200] flex items-center justify-between gap-4 bg-amber-500/10 border-b border-amber-500/30 px-4 py-2 text-sm text-amber-600 dark:text-amber-400">
      <span>
        ⚠ {beforeLink}
        <Link to="/settings" className="font-bold underline" onClick={() => setShow(false)}>
          {settingsLabel}
        </Link>
        {afterLink}
      </span>
      <button onClick={() => setShow(false)} className="shrink-0 text-xs opacity-60 hover:opacity-100">✕</button>
    </div>
  );
}

export function AppShell() {
  const navigate = useNavigate();
  const scrollContainerRef = useRef<HTMLElement | null>(null);

  return (
    <div className="h-screen overflow-hidden bg-x-background dark:bg-x-background-dark">
      <NoKeyBanner />
      <ScrollProgress scrollContainerRef={scrollContainerRef} />

      <div className="mx-auto flex h-screen max-w-[1265px] overflow-hidden">
        {/* 左侧导航 */}
        <LeftNav />

        {/* 主内容区 */}
        <main
          ref={scrollContainerRef}
          data-app-scroll-container
          className="relative h-screen min-w-0 flex-1 overflow-y-auto overscroll-contain border-x border-x-border dark:border-x-border-dark"
        >
          <Outlet />
        </main>

        {/* 右侧边栏 */}
        <RightSidebar />
      </div>

      {/* 移动端悬浮按钮 */}
      <FloatingActionButton onClick={() => navigate("/?compose=1")} />
    </div>
  );
}
