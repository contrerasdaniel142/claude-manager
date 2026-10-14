import { useEffect, useRef, useState } from "react";
import TextInput from "./TextInput";

export interface PaletteCommand {
  id: string;
  label: string;
  hint?: string;
  action: () => void;
}

interface Props {
  open: boolean;
  onClose: () => void;
  commands: PaletteCommand[];
  placeholder: string;
}

export default function CommandPalette({ open, onClose, commands, placeholder }: Props) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const q = query.trim().toLowerCase();
  const filtered = q ? commands.filter((c) => c.label.toLowerCase().includes(q)) : commands;

  useEffect(() => {
    if (open) {
      setQuery("");
      setActiveIndex(0);
      // Tauri's webview needs a tick after the overlay mounts before focus sticks.
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  if (!open) return null;

  const run = (cmd: PaletteCommand) => {
    onClose();
    cmd.action();
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-start justify-center pt-24" onClick={onClose}>
      <div className="fixed inset-0" style={{ background: "rgba(0,0,0,0.4)" }} />
      <div
        className="relative w-[38rem] rounded-xl border shadow-2xl overflow-hidden"
        style={{ background: "var(--bg-elevated)", borderColor: "var(--border)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <TextInput
          ref={inputRef}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={placeholder}
          className="w-full border-0 rounded-none"
          style={{ borderBottom: "1px solid var(--border)" }}
          onKeyDown={(e) => {
            if (e.key === "Escape") onClose();
            else if (e.key === "ArrowDown") {
              e.preventDefault();
              setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setActiveIndex((i) => Math.max(i - 1, 0));
            } else if (e.key === "Enter" && filtered[activeIndex]) {
              run(filtered[activeIndex]);
            }
          }}
        />
        <div className="max-h-80 overflow-y-auto">
          {filtered.map((c, i) => (
            <button
              key={c.id}
              className="w-full text-left px-3 py-2 text-sm flex items-center justify-between gap-3"
              style={{
                background: i === activeIndex ? "var(--accent-soft)" : "transparent",
                color: i === activeIndex ? "var(--accent)" : "var(--text)",
              }}
              onMouseEnter={() => setActiveIndex(i)}
              onClick={() => run(c)}
            >
              <span className="truncate">{c.label}</span>
              {c.hint && (
                <span className="text-xs shrink-0" style={{ color: "var(--text-muted)" }}>
                  {c.hint}
                </span>
              )}
            </button>
          ))}
          {filtered.length === 0 && (
            <div className="px-3 py-3 text-sm" style={{ color: "var(--text-muted)" }}>
              —
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
