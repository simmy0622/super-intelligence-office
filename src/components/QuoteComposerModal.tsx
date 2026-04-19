import { FormEvent, useEffect, useState, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";


interface QuoteComposerModalProps {
  open: boolean;
  targetLabel: string;
  submitting?: boolean;
  onClose: () => void;
  onSubmit: (quoteBody: string | null) => Promise<void>;
}

export function QuoteComposerModal({
  open,
  targetLabel,
  submitting = false,
  onClose,
  onSubmit,
}: QuoteComposerModalProps) {
  const [quoteBody, setQuoteBody] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!open) {
      setQuoteBody("");
    } else {
      // Auto focus when opened
      setTimeout(() => textareaRef.current?.focus(), 100);
    }
  }, [open]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSubmit(quoteBody.trim() || null);
  }

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setQuoteBody(e.target.value);
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
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
            className="fixed inset-0 z-[80] bg-black/60 backdrop-blur-sm"
          />

          {/* Modal */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
            className="fixed inset-0 z-[90] flex items-center justify-center p-4"
          >
            <div className="w-full max-w-lg overflow-hidden rounded-2xl bg-x-background dark:bg-x-surface-dark shadow-modal">
              {/* Header */}
              <div className="flex items-center justify-between border-b border-x-border dark:border-x-border-dark px-4 py-3">
                <button
                  onClick={onClose}
                  className="rounded-full p-2 transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
                >
                  <svg viewBox="0 0 24 24" className="h-5 w-5 text-x-text dark:text-x-text-dark" fill="currentColor">
                    <path d="M10.59 12L4.54 5.96l1.42-1.42L12 10.59l6.04-6.05 1.42 1.42L13.41 12l6.05 6.04-1.42 1.42L12 13.41l-6.04 6.05-1.42-1.42L10.59 12z" />
                  </svg>
                </button>
                <h3 className="font-bold text-x-text dark:text-x-text-dark">Quote Post</h3>
                <div className="w-10" />
              </div>

              {/* Content */}
              <form onSubmit={handleSubmit}>
                <div className="px-4 py-4">
                  {/* Quoting info */}
                  <div className="mb-4 flex items-center gap-2 text-sm text-x-text-secondary">
                    <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
                      <path d="M4.5 3.88l4.432 4.14-1.364 1.46L5.5 7.55V16c0 1.1.896 2 2 2H13v2H7.5c-2.209 0-4-1.79-4-4V7.55L1.432 9.48.068 8.02 4.5 3.88zM16.5 6H11V4h5.5c2.209 0 4 1.79 4 4v8.45l2.068-1.93 1.364 1.46-4.432 4.14-4.432-4.14 1.364-1.46 2.068 1.93V8c0-1.1-.896-2-2-2z" />
                    </svg>
                    <span>Quoting @{targetLabel}</span>
                  </div>

                  {/* Input */}
                  <div className="flex gap-3">
                    <div className="h-10 w-10 flex-shrink-0">
                      <div className="h-full w-full rounded-full bg-gradient-to-br from-x-primary to-purple-500 flex items-center justify-center text-white font-bold">
                        Y
                      </div>
                    </div>
                    <textarea
                      ref={textareaRef}
                      value={quoteBody}
                      onChange={handleInput}
                      placeholder="Add a comment..."
                      rows={3}
                      className="min-w-0 flex-1 resize-none border-none bg-transparent text-xl leading-normal outline-none placeholder:text-x-text-secondary text-x-text dark:text-x-text-dark"
                    />
                  </div>
                </div>

                {/* Footer */}
                <div className="flex items-center justify-between border-t border-x-border dark:border-x-border-dark px-4 py-3">
                  <div className="flex items-center gap-1 text-x-primary">
                    <button type="button" className="rounded-full p-2 transition-colors hover:bg-x-primary/10">
                      <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
                        <path d="M3 5.5C3 4.119 4.119 3 5.5 3h13C19.881 3 21 4.119 21 5.5v13c0 1.381-1.119 2.5-2.5 2.5h-13C4.119 21 3 19.881 3 18.5v-13zM5.5 5c-.276 0-.5.224-.5.5v9.086l3-3 3 3 5-5 3 3V5.5c0-.276-.224-.5-.5-.5h-13zM19 15.414l-3-3-5 5-3-3-3 3V18.5c0 .276.224.5.5.5h13c.276 0 .5-.224.5-.5v-3.086zM9.75 7.75a1.25 1.25 0 11-2.5 0 1.25 1.25 0 012.5 0z" />
                      </svg>
                    </button>
                    <button type="button" className="rounded-full p-2 transition-colors hover:bg-x-primary/10">
                      <svg viewBox="0 0 24 24" className="h-5 w-5" fill="currentColor">
                        <path d="M12 7c-1.1 0-2 .9-2 2v3H8v2h3v6h2v-6h3v-2h-3V9.5c0-.275.225-.5.5-.5h1.5V7h-1.5z" />
                      </svg>
                    </button>
                  </div>

                  <div className="flex items-center gap-3">
                    <button
                      type="button"
                      onClick={() => onSubmit(null)}
                      disabled={submitting}
                      className="rounded-full px-4 py-2 text-sm font-bold text-x-text dark:text-x-text-dark transition-colors hover:bg-x-surface-hover dark:hover:bg-x-surface-hover-dark"
                    >
                      Repost without commenting
                    </button>
                    <button
                      type="submit"
                      disabled={submitting}
                      className="rounded-full bg-x-primary px-5 py-2 text-sm font-bold text-white transition-colors hover:bg-x-primary-hover disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {submitting ? "Posting..." : "Post"}
                    </button>
                  </div>
                </div>
              </form>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
