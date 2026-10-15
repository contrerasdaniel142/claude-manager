import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { Category, Scope, SearchResult } from "../types";
import { useLanguage } from "./LanguageProvider";
import TextInput from "./TextInput";

interface Props {
  open: boolean;
  scope: Scope;
  onClose: () => void;
  onOpenResult: (category: Category, id: string) => void;
}

export default function GlobalSearch({ open, scope, onClose, onOpenResult }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const { t } = useLanguage();

  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setActiveIndex(0);
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const q = query.trim();
    if (!q) {
      setResults([]);
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      api.searchEverything(scope, q).then((r) => {
        if (!cancelled) {
          setResults(r);
          setActiveIndex(0);
        }
      });
    }, 150);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query, scope, open]);

  if (!open) return null;

  const openAt = (i: number) => {
    const r = results[i];
    if (!r) return;
    onClose();
    onOpenResult(r.category, r.id);
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
          placeholder={t.globalSearch.placeholder}
          className="w-full border-0 rounded-none"
          style={{ borderBottom: "1px solid var(--border)" }}
          onKeyDown={(e) => {
            if (e.key === "Escape") onClose();
            else if (e.key === "ArrowDown") {
              e.preventDefault();
              setActiveIndex((i) => Math.min(i + 1, results.length - 1));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setActiveIndex((i) => Math.max(i - 1, 0));
            } else if (e.key === "Enter") {
              openAt(activeIndex);
            }
          }}
        />
        <div className="max-h-96 overflow-y-auto">
          {results.map((r, i) => (
            <button
              key={`${r.category}-${r.id}`}
              className="w-full text-left px-3 py-2 text-sm block cursor-pointer"
              style={{ background: i === activeIndex ? "var(--accent-soft)" : "transparent" }}
              onMouseEnter={() => setActiveIndex(i)}
              onClick={() => openAt(i)}
            >
              <div className="flex items-center justify-between gap-2">
                <span className="truncate font-medium" style={{ color: i === activeIndex ? "var(--accent)" : "var(--text)" }}>
                  {r.title}
                </span>
                <span className="text-[10px] uppercase shrink-0" style={{ color: "var(--text-muted)" }}>
                  {t.app.categories[r.category]}
                </span>
              </div>
              <div className="truncate text-xs" style={{ color: "var(--text-muted)" }}>
                {r.snippet}
              </div>
            </button>
          ))}
          {query.trim() && results.length === 0 && (
            <div className="px-3 py-3 text-sm" style={{ color: "var(--text-muted)" }}>
              {t.globalSearch.noneFound}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
