import { NavLink, useNavigate } from "react-router-dom";
import { useEffect, useState } from "react";
import { createPortal } from "react-dom";

import { cn } from "../lib/utils";
import { getProfileOverride, unreadNotificationCount } from "../lib/client";
import { useLanguage } from "../lib/language";
import { useSalon } from "../lib/salon-context";
import leftNavLogo from "../assets/left-nav-logo.png";
import { AnimatedButton } from "./AnimatedButton";
import { Avatar } from "./Avatar";
import { CreateSalonModal } from "./CreateSalonModal";
import { SettingsGearIcon } from "./SettingsGearIcon";

function HomeIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2.25" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
      <path d="M9 22V12h6v10" />
    </svg>
  );
}

function NotificationIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="currentColor">
      <path d="M19.993 9.042C19.48 5.017 16.054 2 11.996 2s-7.49 3.021-7.999 7.051L2.866 18H7.1c.463 2.282 2.481 4 4.9 4s4.437-1.718 4.9-4h4.236l-1.143-8.958zM12 20c-1.306 0-2.417-.835-2.829-2h5.658c-.412 1.165-1.523 2-2.829 2zm-6.866-4l.847-6.698C6.36 6.272 8.941 4 11.996 4s5.643 2.277 6.013 5.295L18.864 16H5.134z" />
    </svg>
  );
}

function SearchIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="7" />
      <line x1="21" y1="21" x2="16" y2="16" />
    </svg>
  );
}

function ProfileIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="currentColor">
      <path d="M5.651 19h12.698c-.337-1.8-1.023-3.21-1.945-4.19C15.318 13.65 13.838 13 12 13s-3.317.65-4.404 1.81c-.922.98-1.608 2.39-1.945 4.19zm.486-5.56C7.627 11.85 9.68 11 12 11s4.373.85 5.863 2.44c1.477 1.58 2.366 3.8 2.632 6.46l.11 1.1H3.395l.11-1.1c.266-2.66 1.155-4.88 2.632-6.46zM12 4c-1.105 0-2 .9-2 2s.895 2 2 2 2-.9 2-2-.895-2-2-2zM8 6c0-2.21 1.791-4 4-4s4 1.79 4 4-1.791 4-4 4-4-1.79-4-4z" />
    </svg>
  );
}

function SettingsIcon({ className }: { className?: string }) {
  return <SettingsGearIcon className={className} />;
}

function LanguageIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2.1" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 5h10" />
      <path d="M9 3v2c0 4.97-2.239 9.64-6 13" />
      <path d="M6 9c1.029 2.197 2.589 4.27 4.68 6.12" />
      <path d="M14 19h6" />
      <path d="M17 5l4 14" />
      <path d="M13 19l4-14" />
    </svg>
  );
}

export function LeftNav() {
  const navigate = useNavigate();
  const { salons, refreshSalons, setActiveSalonId } = useSalon();
  const { language, setLanguage, t } = useLanguage();
  const [notifCount, setNotifCount] = useState(0);
  const [userProfile, setUserProfile] = useState(() => getProfileOverride("You"));
  const [showCreateSalon, setShowCreateSalon] = useState(false);
  const [showLogoPreview, setShowLogoPreview] = useState(false);

  useEffect(() => {
    const poll = () => {
      unreadNotificationCount()
        .then(setNotifCount)
        .catch(() => {});
    };
    poll();
    const interval = setInterval(poll, 15000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const handler = () => setUserProfile(getProfileOverride("You"));
    window.addEventListener("profile-updated", handler);
    return () => window.removeEventListener("profile-updated", handler);
  }, []);

  useEffect(() => {
    if (!showLogoPreview) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setShowLogoPreview(false);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showLogoPreview]);

  const navItems = [
    { to: "/", label: t("nav.home"), icon: HomeIcon },
    { to: "/search", label: t("nav.search"), icon: SearchIcon },
    { to: "/notifications", label: t("nav.notifications"), icon: NotificationIcon },
    { to: "/profile/You", label: t("nav.profile"), icon: ProfileIcon },
    { to: "/settings", label: t("nav.settings"), icon: SettingsIcon },
  ];

  const items = navItems.map((item) =>
    item.to === "/notifications" ? { ...item, badge: notifCount || undefined } : item,
  );

  const logoPreview = showLogoPreview && typeof document !== "undefined"
    ? createPortal(
        <div
          role="dialog"
          aria-modal="true"
          aria-label="超级智能办公室大图"
          className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/72 p-6 backdrop-blur-sm"
          onClick={() => setShowLogoPreview(false)}
        >
          <div className="relative max-h-[88vh] max-w-[88vw]" onClick={(event) => event.stopPropagation()}>
            <button
              type="button"
              onClick={() => setShowLogoPreview(false)}
              className="absolute -right-3 -top-3 z-10 flex h-9 w-9 items-center justify-center rounded-full bg-white text-lg font-bold text-x-text shadow-lg transition-colors hover:bg-x-surface-hover"
              aria-label="关闭大图"
            >
              ×
            </button>
            <img
              src={leftNavLogo}
              alt="超级智能办公室"
              className="max-h-[88vh] max-w-[88vw] rounded-lg bg-white object-contain shadow-[0_28px_90px_rgba(0,0,0,0.42)]"
            />
          </div>
        </div>,
        document.body,
      )
    : null;

  return (
    <>
    <aside className="sticky top-0 hidden h-screen w-[88px] flex-col items-center overflow-y-auto px-3 py-4 md:flex xl:w-[275px] xl:items-stretch">
      {/* Logo */}
      <button
        type="button"
        onClick={() => setShowLogoPreview(true)}
        className="mb-4 flex items-center justify-center rounded-lg p-1 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark xl:justify-start xl:px-2 xl:py-1"
        aria-label="查看超级智能办公室大图"
      >
        <img
          src={leftNavLogo}
          alt="超级智能办公室"
          className="h-[96px] w-[96px] rounded-lg object-cover xl:hidden"
        />
        <img
          src={leftNavLogo}
          alt="超级智能办公室"
          className="hidden h-[136px] w-full max-w-[300px] object-contain xl:block"
        />
      </button>

      {/* 导航项 */}
      <nav className="flex w-full flex-col items-center gap-1 xl:items-stretch">
        {items.map((item) => {
          return (
            <NavLink
              key={item.label}
              to={item.to}
              aria-label={item.label}
              className={({ isActive }) =>
                cn(
                  "group relative flex w-fit items-center gap-4 rounded-full p-3 text-xl transition-colors xl:w-full xl:px-4 xl:py-3",
                  isActive
                    ? "font-bold text-x-text dark:text-x-text-dark"
                    : "font-normal text-x-text dark:text-x-text-dark hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
                )
              }
            >
              <div
                className="relative rounded-full transition-colors"
              >
                <item.icon className="h-7 w-7 transition-transform group-hover:scale-110" />
                {"badge" in item && item.badge && (
                  <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-x-primary text-xs font-bold text-white">
                    {item.badge}
                  </span>
                )}
              </div>
              <span className="hidden xl:inline">{item.label}</span>
            </NavLink>
          );
        })}
      </nav>

      <div className="mt-1 flex w-full flex-col items-center xl:items-stretch">
        <button
          type="button"
          onClick={() => setLanguage(language === "zh" ? "en" : "zh")}
          aria-label={t("nav.language")}
          title={t("nav.language")}
          className="group relative flex w-fit items-center gap-4 rounded-full p-3 text-xl transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark xl:w-full xl:px-4 xl:py-3"
        >
          <div className="relative rounded-full transition-colors">
            <LanguageIcon className="h-7 w-7 transition-transform group-hover:scale-110" />
          </div>
          <span className="hidden xl:inline">{t("nav.language")}</span>
          <span className="hidden text-sm font-bold text-x-text-secondary xl:ml-auto xl:inline">
            {language === "zh" ? t("nav.chinese") : t("nav.english")}
          </span>
        </button>
      </div>

      {/* Create salon + Post 按钮 */}
      <div className="mt-4 flex w-full flex-col items-center gap-3 xl:items-stretch">
        <AnimatedButton
          size="lg"
          disabled={salons.length >= 3}
          className="hidden w-full xl:flex xl:justify-center"
          onClick={() => setShowCreateSalon(true)}
        >
          {t("nav.createSalon")}
        </AnimatedButton>
        <AnimatedButton
          size="md"
          disabled={salons.length >= 3}
          className="flex h-14 w-14 items-center justify-center p-0 xl:hidden"
          onClick={() => setShowCreateSalon(true)}
        >
          +
        </AnimatedButton>

        <AnimatedButton size="lg" className="hidden w-full xl:flex xl:justify-center" onClick={() => navigate("/?compose=1")}>
          {t("nav.post")}
        </AnimatedButton>
        <AnimatedButton
          size="md"
          className="flex h-14 w-14 items-center justify-center p-0 xl:hidden"
          onClick={() => navigate("/?compose=1")}
        >
          <svg viewBox="0 0 24 24" className="h-6 w-6" fill="currentColor">
            <path d="M23 3c-6.62-.1-10.38 2.421-13.05 6.03C7.29 12.61 6 17.331 6 22h2c0-1.007.07-2.012.19-3H12c4.1 0 7.48-3.082 7.94-7.054C22.79 10.147 23.17 6.359 23 3zm-7 8h-1.5v2H16c.63-.016 1.2-.08 1.72-.188C16.95 15.24 14.68 17 12 17H8.55c.57-2.512 1.57-4.851 3-6.78 2.16-2.912 5.29-4.911 9.45-5.187C20.95 8.079 19.9 11 16 11zM4 9V6H1V4h3V1h2v3h3v2H6v3H4z" />
          </svg>
        </AnimatedButton>
      </div>

      {/* 用户卡片 */}
      <div className="mt-auto flex w-full justify-center xl:justify-start">
        <NavLink
          to="/profile/You"
          className="flex w-fit items-center gap-3 rounded-full p-3 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark xl:w-full"
        >
          <Avatar
            seed={userProfile?.avatar || "You"}
            label={userProfile?.displayName || "You"}
            size="sm"
          />
          <div className="hidden min-w-0 flex-1 text-left xl:block">
            <div className="truncate font-bold text-x-text dark:text-x-text-dark">
              {userProfile?.displayName || "You"}
            </div>
            <div className="truncate text-sm text-x-text-secondary">@You</div>
          </div>
        </NavLink>
      </div>
      <CreateSalonModal
        open={showCreateSalon}
        onClose={() => setShowCreateSalon(false)}
        onCreated={(salon) => {
          void refreshSalons();
          setActiveSalonId(salon.id);
          navigate("/");
        }}
      />
    </aside>
    {logoPreview}
    </>
  );
}
