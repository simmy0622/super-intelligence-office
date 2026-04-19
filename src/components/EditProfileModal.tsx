import { useEffect, useState, useRef, ChangeEvent } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useLanguage } from "../lib/language";
import { Avatar } from "./Avatar";

type ResizeOptions = {
  maxWidth: number;
  maxHeight: number;
  quality: number;
};

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      if (typeof reader.result === "string") {
        resolve(reader.result);
      } else {
        reject(new Error("Could not read image file."));
      }
    };
    reader.onerror = () => reject(reader.error ?? new Error("Could not read image file."));
    reader.readAsDataURL(file);
  });
}

async function resizeImageFile(file: File, options: ResizeOptions): Promise<string> {
  if (!file.type.startsWith("image/") || file.type === "image/svg+xml") {
    return readFileAsDataUrl(file);
  }

  const objectUrl = URL.createObjectURL(file);
  try {
    const image = new Image();
    image.decoding = "async";
    const loaded = new Promise<void>((resolve, reject) => {
      image.onload = () => resolve();
      image.onerror = () => reject(new Error("Could not load image file."));
    });
    image.src = objectUrl;
    await loaded;

    const naturalWidth = image.naturalWidth || image.width;
    const naturalHeight = image.naturalHeight || image.height;
    if (!naturalWidth || !naturalHeight) return readFileAsDataUrl(file);

    const scale = Math.min(1, options.maxWidth / naturalWidth, options.maxHeight / naturalHeight);
    const width = Math.max(1, Math.round(naturalWidth * scale));
    const height = Math.max(1, Math.round(naturalHeight * scale));
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;

    const context = canvas.getContext("2d");
    if (!context) return readFileAsDataUrl(file);

    context.fillStyle = "#ffffff";
    context.fillRect(0, 0, width, height);
    context.drawImage(image, 0, 0, width, height);

    return canvas.toDataURL("image/jpeg", options.quality);
  } finally {
    URL.revokeObjectURL(objectUrl);
  }
}


interface EditProfileModalProps {
  open: boolean;
  onClose: () => void;
  currentAvatar?: string | null;
  currentBanner?: string | null;
  fallbackAvatarSeed?: string | null;
  displayName: string;
  bio: string;
  mode?: "full" | "avatar";
  title?: string;
  saveLabel?: string;
  onSave: (data: {
    avatar?: string;
    banner?: string;
    displayName: string;
    bio: string;
  }) => Promise<void>;
}

export function EditProfileModal({
  open,
  onClose,
  currentAvatar,
  currentBanner,
  fallbackAvatarSeed,
  displayName: initialName,
  bio: initialBio,
  mode = "full",
  title = "Edit profile",
  saveLabel = "Save",
  onSave,
}: EditProfileModalProps) {
  const { t } = useLanguage();
  const [displayName, setDisplayName] = useState(initialName);
  const [bio, setBio] = useState(initialBio);
  const [avatar, setAvatar] = useState<string | undefined>(currentAvatar || undefined);
  const [banner, setBanner] = useState<string | undefined>(currentBanner || undefined);
  const [saving, setSaving] = useState(false);
  const [imageError, setImageError] = useState<string | null>(null);
  const avatarInputRef = useRef<HTMLInputElement>(null);
  const bannerInputRef = useRef<HTMLInputElement>(null);
  const avatarPreviewSeed = avatar ?? fallbackAvatarSeed ?? displayName ?? "Profile";
  const showFullEditor = mode === "full";

  useEffect(() => {
    if (!open) return;
    setDisplayName(initialName);
    setBio(initialBio);
    setAvatar(currentAvatar || undefined);
    setBanner(currentBanner || undefined);
    setImageError(null);
  }, [open, currentAvatar, currentBanner, initialName, initialBio]);

  const handleAvatarChange = async (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setImageError(null);
      try {
        setAvatar(await resizeImageFile(file, { maxWidth: 512, maxHeight: 512, quality: 0.85 }));
      } catch (error) {
        setImageError(error instanceof Error ? error.message : "Could not read image file.");
      } finally {
        e.currentTarget.value = "";
      }
    }
  };

  const handleBannerChange = async (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setImageError(null);
      try {
        setBanner(await resizeImageFile(file, { maxWidth: 1600, maxHeight: 640, quality: 0.82 }));
      } catch (error) {
        setImageError(error instanceof Error ? error.message : "Could not read image file.");
      } finally {
        e.currentTarget.value = "";
      }
    }
  };

  const handleSubmit = async () => {
    setSaving(true);
    setImageError(null);
    try {
      await onSave({
        avatar,
        banner,
        displayName: displayName.trim() || initialName,
        bio: bio.trim(),
      });
      onClose();
    } catch (error) {
      setImageError(error instanceof Error ? error.message : "Failed to save profile.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <AnimatePresence>
      {open && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 z-[100] bg-black/60 backdrop-blur-sm"
          />

          {/* Modal */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
            className="fixed inset-0 z-[110] flex items-center justify-center p-4"
          >
            <div className="w-full max-w-lg overflow-hidden rounded-2xl bg-x-background dark:bg-x-surface-dark shadow-modal max-h-[90vh] overflow-y-auto">
              {/* Header */}
              <div className="sticky top-0 z-10 flex items-center justify-between border-b border-x-border dark:border-x-border-dark bg-x-background dark:bg-x-surface-dark px-4 py-3">
                <div className="flex items-center gap-4">
                  <button
                    onClick={onClose}
                    className="rounded-full p-2 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
                  >
                    <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text dark:text-x-text-dark" fill="currentColor">
                      <path d="M10.59 12L4.54 5.96l1.42-1.42L12 10.59l6.04-6.05 1.42 1.42L13.41 12l6.05 6.04-1.42 1.42L12 13.41l-6.04 6.05-1.42-1.42L10.59 12z" />
                    </svg>
                  </button>
                  <h2 className="text-xl font-bold text-x-text dark:text-x-text-dark">{title}</h2>
                </div>
                <button
                  onClick={handleSubmit}
                  disabled={saving}
                  className="rounded-full bg-x-text dark:bg-x-text-dark px-5 py-2 text-sm font-bold text-white dark:text-black transition-colors hover:bg-x-text/90 dark:hover:bg-x-text-dark/90 disabled:opacity-50"
                >
                  {saving ? t("common.saving") : saveLabel}
                </button>
              </div>

              {/* Content */}
              <div className="relative">
                {showFullEditor && (
                  <>
                    {/* Banner */}
                    <div className="relative h-48 bg-gradient-to-r from-x-primary via-purple-500 to-pink-500">
                      {banner && (
                        <img
                          src={banner}
                          alt="Banner"
                          className="h-full w-full object-cover"
                        />
                      )}
                      <div className="absolute inset-0 bg-black/30 flex items-center justify-center">
                        <button
                          onClick={() => bannerInputRef.current?.click()}
                          className="rounded-full bg-black/50 p-3 text-white transition-colors hover:bg-black/70"
                        >
                          <svg viewBox="0 0 24 24" className="h-6 w-6" fill="currentColor">
                            <path d="M3 5.5C3 4.119 4.119 3 5.5 3h13C19.881 3 21 4.119 21 5.5v13c0 1.381-1.119 2.5-2.5 2.5h-13C4.119 21 3 19.881 3 18.5v-13zM5.5 5c-.276 0-.5.224-.5.5v9.086l3-3 3 3 5-5 3 3V5.5c0-.276-.224-.5-.5-.5h-13zM19 15.414l-3-3-5 5-3-3-3 3V18.5c0 .276.224.5.5.5h13c.276 0 .5-.224.5-.5v-3.086zM9.75 7.75a1.25 1.25 0 11-2.5 0 1.25 1.25 0 012.5 0z" />
                          </svg>
                        </button>
                        {banner && (
                          <button
                            onClick={() => setBanner(undefined)}
                            className="absolute top-2 right-2 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
                          >
                            <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
                              <path d="M10.59 12L4.54 5.96l1.42-1.42L12 10.59l6.04-6.05 1.42 1.42L13.41 12l6.05 6.04-1.42 1.42L12 13.41l-6.04 6.05-1.42-1.42L10.59 12z" />
                            </svg>
                          </button>
                        )}
                      </div>
                      <input
                        ref={bannerInputRef}
                        type="file"
                        accept="image/*"
                        onChange={handleBannerChange}
                        className="hidden"
                      />
                    </div>

                    {/* Avatar */}
                    <div className="absolute -bottom-12 left-4">
                      <div className="relative">
                        <div className="h-24 w-24 rounded-full border-4 border-x-background dark:border-x-background-dark bg-x-surface-hover overflow-hidden">
                          <Avatar seed={avatarPreviewSeed} label={displayName || "Profile"} className="h-full w-full border-0" />
                        </div>
                        <button
                          onClick={() => avatarInputRef.current?.click()}
                          className="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 text-white opacity-0 transition-opacity hover:opacity-100"
                        >
                          <svg viewBox="0 0 24 24" className="h-6 w-6" fill="currentColor">
                            <path d="M3 5.5C3 4.119 4.119 3 5.5 3h13C19.881 3 21 4.119 21 5.5v13c0 1.381-1.119 2.5-2.5 2.5h-13C4.119 21 3 19.881 3 18.5v-13zM5.5 5c-.276 0-.5.224-.5.5v9.086l3-3 3 3 5-5 3 3V5.5c0-.276-.224-.5-.5-.5h-13zM19 15.414l-3-3-5 5-3-3-3 3V18.5c0 .276.224.5.5.5h13c.276 0 .5-.224.5-.5v-3.086zM9.75 7.75a1.25 1.25 0 11-2.5 0 1.25 1.25 0 012.5 0z" />
                          </svg>
                        </button>
                        <input
                          ref={avatarInputRef}
                          type="file"
                          accept="image/*"
                          onChange={handleAvatarChange}
                          className="hidden"
                        />
                      </div>
                    </div>
                  </>
                )}

                {!showFullEditor && (
                  <div className="px-4 pt-8">
                    <div className="mx-auto relative h-28 w-28 rounded-full border-4 border-x-background dark:border-x-background-dark bg-x-surface-hover overflow-hidden">
                      <Avatar seed={avatarPreviewSeed} label={displayName || "Profile"} className="h-full w-full border-0" />
                      <button
                        onClick={() => avatarInputRef.current?.click()}
                        className="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 text-white opacity-0 transition-opacity hover:opacity-100"
                      >
                        <svg viewBox="0 0 24 24" className="h-6 w-6" fill="currentColor">
                          <path d="M3 5.5C3 4.119 4.119 3 5.5 3h13C19.881 3 21 4.119 21 5.5v13c0 1.381-1.119 2.5-2.5 2.5h-13C4.119 21 3 19.881 3 18.5v-13zM5.5 5c-.276 0-.5.224-.5.5v9.086l3-3 3 3 5-5 3 3V5.5c0-.276-.224-.5-.5-.5h-13zM19 15.414l-3-3-5 5-3-3-3 3V18.5c0 .276.224.5.5.5h13c.276 0 .5-.224.5-.5v-3.086zM9.75 7.75a1.25 1.25 0 11-2.5 0 1.25 1.25 0 012.5 0z" />
                        </svg>
                      </button>
                      <input
                        ref={avatarInputRef}
                        type="file"
                        accept="image/*"
                        onChange={handleAvatarChange}
                        className="hidden"
                      />
                    </div>
                  </div>
                )}
              </div>

              {/* Form */}
              <div className={`${showFullEditor ? "mt-16" : "mt-6"} px-4 pb-6 space-y-4`}>
                {imageError && (
                  <p className="rounded-xl bg-red-500/10 px-3 py-2 text-sm font-medium text-red-500">
                    {imageError}
                  </p>
                )}

                {showFullEditor ? (
                  <>
                    {/* Display Name */}
                    <div>
                      <label className="mb-1 block text-sm font-medium text-x-text-secondary">
                        {t("profile.displayNameLabel")}
                      </label>
                      <input
                        value={displayName}
                        onChange={(e) => setDisplayName(e.target.value)}
                        maxLength={50}
                        className="w-full rounded-xl border border-x-border dark:border-x-border-dark bg-x-background dark:bg-x-surface-dark px-4 py-3 text-x-text dark:text-x-text-dark outline-none focus:border-x-primary focus:ring-2 focus:ring-x-primary/20 transition-all"
                      />
                      <p className="mt-1 text-right text-xs text-x-text-secondary">
                        {displayName.length}/50
                      </p>
                    </div>

                    {/* Bio */}
                    <div>
                      <label className="mb-1 block text-sm font-medium text-x-text-secondary">
                        {t("profile.bioLabel")}
                      </label>
                      <textarea
                        value={bio}
                        onChange={(e) => setBio(e.target.value)}
                        maxLength={160}
                        rows={3}
                        className="w-full resize-none rounded-xl border border-x-border dark:border-x-border-dark bg-x-background dark:bg-x-surface-dark px-4 py-3 text-x-text dark:text-x-text-dark outline-none focus:border-x-primary focus:ring-2 focus:ring-x-primary/20 transition-all"
                      />
                      <p className="mt-1 text-right text-xs text-x-text-secondary">
                        {bio.length}/160
                      </p>
                    </div>
                  </>
                ) : (
                  <p className="text-center text-sm leading-6 text-x-text-secondary">
                    {t("profile.uploadAvatarHint")}
                  </p>
                )}
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
