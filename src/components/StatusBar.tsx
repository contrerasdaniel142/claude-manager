import { useState } from "react";
import type { Category, Scope } from "../types";
import { useLanguage } from "./LanguageProvider";

export interface ProblemItem {
  category: Category;
  title: string;
  message: string;
}

interface Props {
  scope: Scope;
  problems: ProblemItem[];
  onNavigate: (category: Category) => void;
}

export default function StatusBar({ scope, problems, onNavigate }: Props) {
  const [open, setOpen] = useState(false);
  const { t } = useLanguage();

  return (
    <div
      className="h-6 flex items-center justify-between px-3 text-[11px] shrink-0 border-t relative select-none"
      style={{ background: "var(--bg-sidebar)", color: "#9b9ba6", borderColor: "var(--border)" }}
    >
      <div>{scope === "project" ? t.app.scopeProject : t.app.scopeGlobal}</div>

      {problems.length > 0 && (
        <button
          onClick={() => setOpen((v) => !v)}
          className="flex items-center gap-1 hover:opacity-80 cursor-pointer"
          style={{ color: "var(--warning)" }}
        >
          <span>&#9888;</span>
          {problems.length}
        </button>
      )}

      {open && problems.length > 0 && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <div
            className="absolute bottom-6 right-2 z-50 w-80 max-h-64 overflow-y-auto rounded-lg border shadow-lg text-xs"
            style={{ background: "var(--bg-elevated)", borderColor: "var(--border)", color: "var(--text)" }}
          >
            {problems.map((p, i) => (
              <button
                key={i}
                className="w-full text-left px-3 py-2 border-b hover:opacity-80 block cursor-pointer"
                style={{ borderColor: "var(--border)" }}
                onClick={() => {
                  setOpen(false);
                  onNavigate(p.category);
                }}
              >
                <div className="font-medium truncate">{p.title}</div>
                <div className="truncate" style={{ color: "var(--text-muted)" }}>
                  {p.message}
                </div>
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
