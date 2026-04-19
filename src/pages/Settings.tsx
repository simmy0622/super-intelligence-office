import { FormEvent, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  getApiKeyStatus,
  getSettings,
  listActors,
  setApiKey,
  setSettings,
  updateActor,
  type Actor,
  type SettingEntry,
} from "../lib/client";
import { useLanguage } from "../lib/language";
import { cn } from "../lib/utils";

// ── toast ─────────────────────────────────────────────────────────────────────

function Toast({ message, onDone }: { message: string; onDone: () => void }) {
  useEffect(() => {
    const t = setTimeout(onDone, 2500);
    return () => clearTimeout(t);
  }, [onDone]);
  return (
    <div className="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-full bg-emerald-500 px-5 py-2 text-sm font-bold text-white shadow-lg">
      {message}
    </div>
  );
}

// ── api key section ───────────────────────────────────────────────────────────

function ApiKeySection() {
  const { t } = useLanguage();
  const [key, setKey] = useState("");
  const [configured, setConfigured] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [show, setShow] = useState(false);

  useEffect(() => {
    void getApiKeyStatus("deepseek").then((s) => setConfigured(s.configured));
  }, []);

  const handleSave = async () => {
    if (!key.trim()) return;
    setSaving(true);
    try {
      await setApiKey("deepseek", key.trim());
      setConfigured(true);
      setKey("");
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="mb-6">
      <div className="mb-2">
        <span className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
          {t("settings.providerKeys")}
        </span>
      </div>
      <div className="overflow-hidden rounded-2xl border border-x-border dark:border-x-border-dark">
        <div className="space-y-3 px-4 py-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-bold text-x-text dark:text-x-text-dark">{t("settings.deepseek")}</span>
            {configured === true && (
              <span className="text-xs font-bold text-emerald-500">✓ {t("settings.configured")}</span>
            )}
            {configured === false && (
              <span className="text-xs font-bold text-amber-500">{t("common.notSet")}</span>
            )}
          </div>
          <div className="flex gap-2">
            <div className="relative flex-1">
              <input
                type={show ? "text" : "password"}
                value={key}
                onChange={(e) => setKey(e.target.value)}
                placeholder={configured ? t("settings.replaceKeyPlaceholder") : "sk-…"}
                className={cn(inputCls, "pr-14 font-mono text-xs")}
              />
              <button
                type="button"
                onClick={() => setShow((v) => !v)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-x-text-secondary hover:text-x-text"
              >
                {show ? t("common.hide") : t("common.show")}
              </button>
            </div>
            <button
              type="button"
              disabled={!key.trim() || saving}
              onClick={() => void handleSave()}
              className="rounded-full bg-x-primary px-4 py-1.5 text-sm font-bold text-white transition hover:bg-x-primary-hover disabled:cursor-not-allowed disabled:opacity-40"
            >
              {saving ? t("common.saving") : saved ? t("common.saved") : t("common.save")}
            </button>
          </div>
          <a
            href="https://platform.deepseek.com/api_keys"
            target="_blank"
            rel="noreferrer"
            className="block text-xs text-x-primary hover:underline"
          >
            {t("settings.getDeepSeekKey")} →
          </a>
        </div>
      </div>
    </section>
  );
}

// ── agent editor row ──────────────────────────────────────────────────────────

function AgentRow({ actor }: { actor: Actor }) {
  const { t } = useLanguage();
  const [expanded, setExpanded] = useState(false);
  const [draft, setDraft] = useState({
    displayName: actor.displayName,
    bio: actor.bio ?? "",
    specialty: actor.specialty ?? "",
    personaPrompt: actor.personaPrompt ?? "",
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const originalRef = useRef(draft);

  const dirty =
    draft.displayName !== originalRef.current.displayName ||
    draft.bio !== originalRef.current.bio ||
    draft.specialty !== originalRef.current.specialty ||
    draft.personaPrompt !== originalRef.current.personaPrompt;

  const handleSave = async () => {
    setSaving(true);
    try {
      await updateActor(actor.handle, {
        displayName: draft.displayName || undefined,
        bio: draft.bio || null,
        specialty: draft.specialty || null,
        personaPrompt: draft.personaPrompt || null,
      });
      originalRef.current = draft;
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="border-b border-x-border last:border-0 dark:border-x-border-dark">
      <button
        type="button"
        onClick={() => setExpanded((v) => !v)}
        className="flex w-full items-center gap-4 px-4 py-3 text-left transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
      >
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-x-primary/10 text-sm font-extrabold text-x-primary">
          {actor.handle.charAt(0).toUpperCase()}
        </div>
        <div className="min-w-0 flex-1">
          <div className="font-bold text-x-text dark:text-x-text-dark">{actor.displayName}</div>
          <div className="text-xs text-x-text-secondary">@{actor.handle}{actor.specialty ? ` · ${actor.specialty}` : ""}</div>
        </div>
        <svg
          viewBox="0 0 24 24"
          className={cn(
            "h-4 w-4 shrink-0 text-x-text-secondary transition-transform duration-200",
            expanded ? "rotate-180" : "rotate-0",
          )}
          fill="none"
          stroke="currentColor"
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>

      {expanded && (
        <div className="space-y-4 px-4 pb-4 pt-1">
          <Field label={t("settings.displayName")}>
            <input
              value={draft.displayName}
              onChange={(e) => setDraft((d) => ({ ...d, displayName: e.target.value }))}
              className={inputCls}
            />
          </Field>
          <Field label={t("settings.specialty")}>
            <input
              value={draft.specialty}
              onChange={(e) => setDraft((d) => ({ ...d, specialty: e.target.value }))}
              placeholder="e.g. neuroscience / sleep"
              className={inputCls}
            />
          </Field>
          <Field label={t("settings.bio")}>
            <textarea
              rows={2}
              value={draft.bio}
              onChange={(e) => setDraft((d) => ({ ...d, bio: e.target.value }))}
              className={cn(inputCls, "resize-none")}
            />
          </Field>
          <Field label={t("settings.personaPrompt")}>
            <textarea
              rows={5}
              value={draft.personaPrompt}
              onChange={(e) => setDraft((d) => ({ ...d, personaPrompt: e.target.value }))}
              placeholder="System-level persona instructions…"
              className={cn(inputCls, "resize-y font-mono text-xs")}
            />
          </Field>
          {/* read-only schedule info */}
          {(actor.postsPerDay != null || actor.activeHours) && (
            <div className="flex gap-4 text-xs text-x-text-secondary">
              {actor.postsPerDay != null && <span>{t("settings.postsPerDay")}: {actor.postsPerDay}</span>}
              {actor.activeHours && <span>{t("settings.activeHours")}: {actor.activeHours}</span>}
            </div>
          )}
          <div className="flex items-center gap-3 pt-1">
            <button
              type="button"
              disabled={!dirty || saving}
              onClick={() => void handleSave()}
              className="rounded-full bg-x-primary px-4 py-1.5 text-sm font-bold text-white transition hover:bg-x-primary-hover disabled:cursor-not-allowed disabled:opacity-40"
            >
              {saving ? t("common.saving") : saved ? t("common.saved") : t("common.save")}
            </button>
            {dirty && (
              <button
                type="button"
                onClick={() => setDraft(originalRef.current)}
                className="text-sm text-x-text-secondary hover:text-x-text"
              >
                {t("common.discard")}
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// ── shared field wrapper ──────────────────────────────────────────────────────

const inputCls =
  "w-full rounded-xl border border-x-border dark:border-x-border-dark bg-x-background dark:bg-x-background-dark px-3 py-2 text-sm text-x-text dark:text-x-text-dark outline-none focus:border-x-primary focus:ring-2 focus:ring-x-primary/20 transition-all";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label className="mb-1 block text-[11px] font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
        {label}
      </label>
      {children}
    </div>
  );
}

// ── main ──────────────────────────────────────────────────────────────────────

export function Settings() {
  const navigate = useNavigate();
  const { t } = useLanguage();
  const [settings, setLocalSettings] = useState<SettingEntry[]>([]);
  const [agents, setAgents] = useState<Actor[]>([]);
  const [saving, setSaving] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    void getSettings().then(setLocalSettings);
    void listActors().then((actors) => setAgents(actors.filter((a) => a.kind === "agent")));
  }, []);

  function updateSetting(index: number, value: string) {
    setLocalSettings((current) =>
      current.map((entry, i) => (i === index ? { ...entry, value } : entry))
    );
  }

  async function handleSaveSettings(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    try {
      await setSettings(settings);
      setToast(t("settings.settingsSaved"));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div>
      {/* header */}
      <header className="sticky top-0 z-50 glass border-b border-x-border dark:border-x-border-dark">
        <div className="flex h-14 items-center gap-4 px-4">
          <button
            onClick={() => navigate(-1)}
            className="rounded-full p-2 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text dark:text-x-text-dark" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <path d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <h1 className="text-xl font-bold text-x-text dark:text-x-text-dark">{t("settings.title")}</h1>
        </div>
      </header>

      <div className="px-4 py-4">

        {/* ── Provider Keys ────────────────────────────────────── */}
        <ApiKeySection />

        {/* ── Agents ───────────────────────────────────────────── */}
        <section className="mb-6">
          <div className="mb-2 px-0">
            <span className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
              {t("settings.agents")}
            </span>
          </div>
          <div className="overflow-hidden rounded-2xl border border-x-border dark:border-x-border-dark">
            {agents.length === 0 ? (
              <p className="px-4 py-3 text-sm text-x-text-secondary">{t("common.loading")}</p>
            ) : (
              agents.map((agent) => <AgentRow key={agent.id} actor={agent} />)
            )}
          </div>
        </section>

        {/* ── System Config ────────────────────────────────────── */}
        <form onSubmit={handleSaveSettings}>
          <section className="mb-6">
            <div className="mb-2">
              <span className="text-xs font-extrabold uppercase tracking-[0.18em] text-x-text-secondary">
                {t("settings.systemConfig")}
              </span>
            </div>
            <div className="space-y-3">
              {settings.map((entry, index) => (
                <Field key={entry.key} label={entry.key.replace(/_/g, " ")}>
                  <input
                    value={entry.value}
                    onChange={(e) => updateSetting(index, e.target.value)}
                    className={inputCls}
                  />
                </Field>
              ))}
            </div>
          </section>

          <button
            type="submit"
            disabled={saving}
            className="w-full rounded-full bg-x-primary py-3 text-base font-bold text-white shadow-sm transition-colors hover:bg-x-primary-hover disabled:cursor-not-allowed disabled:opacity-50"
          >
            {saving ? t("common.saving") : t("settings.saveConfig")}
          </button>
        </form>

      </div>

      {toast && <Toast message={toast} onDone={() => setToast(null)} />}
    </div>
  );
}
