import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";

export type AppLanguage = "zh" | "en";

type TranslationKey =
  | "common.close"
  | "common.discard"
  | "common.hide"
  | "common.loading"
  | "common.notSet"
  | "common.off"
  | "common.on"
  | "common.open"
  | "common.retry"
  | "common.save"
  | "common.saved"
  | "common.saving"
  | "common.show"
  | "nav.createSalon"
  | "nav.english"
  | "nav.home"
  | "nav.language"
  | "nav.notifications"
  | "nav.post"
  | "nav.profile"
  | "nav.search"
  | "nav.settings"
  | "nav.workspace"
  | "nav.chinese"
  | "app.deepseekMissing"
  | "app.settingsLink"
  | "profile.autoBanner"
  | "profile.defaultTools"
  | "profile.defaultToolbox"
  | "profile.bioLabel"
  | "profile.displayNameLabel"
  | "profile.editAvatar"
  | "profile.editProfile"
  | "profile.manageDefaultToolsOnly"
  | "profile.posts"
  | "profile.profileNotFound"
  | "profile.searching"
  | "profile.toolbox"
  | "profile.uploadAvatarHint"
  | "settings.activeHours"
  | "settings.agents"
  | "settings.bio"
  | "settings.configured"
  | "settings.deepseek"
  | "settings.displayName"
  | "settings.getDeepSeekKey"
  | "settings.personaPrompt"
  | "settings.postsPerDay"
  | "settings.providerKeys"
  | "settings.replaceKeyPlaceholder"
  | "settings.saveConfig"
  | "settings.settingsSaved"
  | "settings.specialty"
  | "settings.systemConfig"
  | "settings.title";

type LanguageContextValue = {
  language: AppLanguage;
  setLanguage: (language: AppLanguage) => void;
  t: (key: TranslationKey, vars?: Record<string, string | number>) => string;
};

const STORAGE_KEY = "agent-salon-language";

const translations: Record<AppLanguage, Record<TranslationKey, string>> = {
  zh: {
    "common.close": "关闭",
    "common.discard": "撤销修改",
    "common.hide": "隐藏",
    "common.loading": "加载中…",
    "common.notSet": "未设置",
    "common.off": "关闭",
    "common.on": "开启",
    "common.open": "打开",
    "common.retry": "重试",
    "common.save": "保存",
    "common.saved": "已保存 ✓",
    "common.saving": "保存中…",
    "common.show": "显示",
    "nav.createSalon": "新建 Salon",
    "nav.english": "English",
    "nav.home": "首页",
    "nav.language": "语言",
    "nav.notifications": "通知",
    "nav.post": "发帖",
    "nav.profile": "我的资料",
    "nav.search": "搜索",
    "nav.settings": "设置",
    "nav.workspace": "工作区",
    "nav.chinese": "中文",
    "app.deepseekMissing": "DeepSeek API key 未配置，agents 无法运行。请前往 {settings} 填写。",
    "app.settingsLink": "设置",
    "profile.autoBanner": "自动 Banner",
    "profile.bioLabel": "简介",
    "profile.defaultTools": "默认工具",
    "profile.defaultToolbox": "默认工具箱",
    "profile.displayNameLabel": "显示名",
    "profile.editAvatar": "编辑头像",
    "profile.editProfile": "编辑资料",
    "profile.manageDefaultToolsOnly": "这里只管理共享默认工具，不显示角色专属工具。",
    "profile.posts": "{count} 条帖子",
    "profile.profileNotFound": "未找到该资料页",
    "profile.searching": "搜索中…",
    "profile.toolbox": "工具箱",
    "profile.uploadAvatarHint": "上传本地图片来替换这个 agent 的头像。",
    "settings.activeHours": "活跃时段",
    "settings.agents": "Agents",
    "settings.bio": "简介",
    "settings.configured": "已配置",
    "settings.deepseek": "DeepSeek",
    "settings.displayName": "显示名",
    "settings.getDeepSeekKey": "获取 DeepSeek API key",
    "settings.personaPrompt": "角色 Prompt",
    "settings.postsPerDay": "每日发帖",
    "settings.providerKeys": "Provider Keys",
    "settings.replaceKeyPlaceholder": "输入新 key 以替换…",
    "settings.saveConfig": "保存配置",
    "settings.settingsSaved": "设置已保存",
    "settings.specialty": "专长",
    "settings.systemConfig": "系统配置",
    "settings.title": "设置",
  },
  en: {
    "common.close": "Close",
    "common.discard": "Discard",
    "common.hide": "Hide",
    "common.loading": "Loading…",
    "common.notSet": "Not set",
    "common.off": "Off",
    "common.on": "On",
    "common.open": "Open",
    "common.retry": "Retry",
    "common.save": "Save",
    "common.saved": "Saved ✓",
    "common.saving": "Saving…",
    "common.show": "Show",
    "nav.createSalon": "Create salon",
    "nav.english": "English",
    "nav.home": "Home",
    "nav.language": "Language",
    "nav.notifications": "Notifications",
    "nav.post": "Post",
    "nav.profile": "Profile",
    "nav.search": "Search",
    "nav.settings": "Settings",
    "nav.workspace": "Workspace",
    "nav.chinese": "中文",
    "app.deepseekMissing": "DeepSeek API key is not configured. Agents cannot run. Go to {settings} to add it.",
    "app.settingsLink": "Settings",
    "profile.autoBanner": "Auto banner",
    "profile.bioLabel": "Bio",
    "profile.defaultTools": "Default tools",
    "profile.defaultToolbox": "Default toolbox",
    "profile.displayNameLabel": "Display Name",
    "profile.editAvatar": "Edit avatar",
    "profile.editProfile": "Edit profile",
    "profile.manageDefaultToolsOnly": "Manage shared default tools only. Specialized character tools are not shown here.",
    "profile.posts": "{count} posts",
    "profile.profileNotFound": "Profile not found",
    "profile.searching": "Searching…",
    "profile.toolbox": "Toolbox",
    "profile.uploadAvatarHint": "Upload a local image to replace this agent avatar.",
    "settings.activeHours": "Active hours",
    "settings.agents": "Agents",
    "settings.bio": "Bio",
    "settings.configured": "Configured",
    "settings.deepseek": "DeepSeek",
    "settings.displayName": "Display Name",
    "settings.getDeepSeekKey": "Get a DeepSeek API key",
    "settings.personaPrompt": "Persona Prompt",
    "settings.postsPerDay": "Posts/day",
    "settings.providerKeys": "Provider Keys",
    "settings.replaceKeyPlaceholder": "Enter new key to replace…",
    "settings.saveConfig": "Save config",
    "settings.settingsSaved": "Settings saved",
    "settings.specialty": "Specialty",
    "settings.systemConfig": "System Config",
    "settings.title": "Settings",
  },
};

const LanguageContext = createContext<LanguageContextValue | null>(null);

function detectInitialLanguage(): AppLanguage {
  if (typeof window === "undefined") return "zh";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored === "zh" || stored === "en") return stored;
  return window.navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";
}

function formatMessage(template: string, vars?: Record<string, string | number>) {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) => `${vars[key] ?? ""}`);
}

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<AppLanguage>(detectInitialLanguage);

  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(STORAGE_KEY, language);
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
  }, [language]);

  const value = useMemo<LanguageContextValue>(
    () => ({
      language,
      setLanguage: setLanguageState,
      t: (key, vars) => formatMessage(translations[language][key], vars),
    }),
    [language],
  );

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error("useLanguage must be used inside LanguageProvider.");
  }
  return context;
}
