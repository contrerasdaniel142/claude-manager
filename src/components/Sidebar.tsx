import { useRef, useState } from "react";
import type { Category, Scope } from "../types";
import { useLanguage } from "./LanguageProvider";

interface Props {
  scope: Scope;
  setScope: (s: Scope) => void;
  category: Category;
  setCategory: (c: Category) => void;
  counts: Record<Category, number>;
  invalidCounts: Record<Category, number>;
  projectRoot: string;
  globalRoot: string;
  disabled: boolean;
}

const WIDTH_KEY = "clman-sidebar-width";
const DEFAULT_WIDTH = 240; // Zed's project_panel.default_width
const MIN_WIDTH = 180;
const MAX_WIDTH = 420;

function clampWidth(w: number): number {
  return Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, w));
}

function loadWidth(): number {
  try {
    const n = Number(localStorage.getItem(WIDTH_KEY));
    return Number.isFinite(n) && n > 0 ? clampWidth(n) : DEFAULT_WIDTH;
  } catch {
    return DEFAULT_WIDTH;
  }
}

export default function Sidebar({
  scope,
  setScope,
  category,
  setCategory,
  counts,
  invalidCounts,
  projectRoot,
  globalRoot,
  disabled,
}: Props) {
  const { t } = useLanguage();
  const [width, setWidth] = useState(loadWidth);
  const widthRef = useRef(width);

  const startResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const onMove = (ev: MouseEvent) => {
      const w = clampWidth(ev.clientX);
      widthRef.current = w;
      setWidth(w);
    };
    const onUp = () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      try {
        localStorage.setItem(WIDTH_KEY, String(widthRef.current));
      } catch {
        // per-viewer convenience only.
      }
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  const CATEGORIES: { id: Category; label: string; showCount?: boolean }[] = [
    { id: "advisor", label: t.app.categories.advisor, showCount: false },
    { id: "steering", label: t.app.categories.steering },
    { id: "rules", label: t.app.categories.rules },
    { id: "hooks", label: t.app.categories.hooks },
    { id: "agents", label: t.app.categories.agents },
    { id: "skills", label: t.app.categories.skills },
    { id: "outputStyles", label: t.app.categories.outputStyles },
    { id: "permissions", label: t.app.categories.permissions },
    { id: "mcpServers", label: t.app.categories.mcpServers },
    { id: "plugins", label: t.app.categories.plugins },
  ];

  return (
    <aside
      className="shrink-0 flex flex-col h-full relative"
      style={{ background: "var(--bg-sidebar)", color: "#e8e8ee", width }}
    >
      <div
        onMouseDown={startResize}
        className="absolute top-0 right-0 h-full w-1 cursor-col-resize z-10 hover:bg-white/10"
      />
      <div className="px-4 py-4 border-b border-white/10 flex items-start justify-between gap-2">
        <div>
          <div className="text-lg font-semibold tracking-tight">{t.app.title}</div>
          <div className="text-xs opacity-50 mt-0.5">{t.app.subtitle}</div>
        </div>
      </div>

      <div className="px-3 pt-3">
        <div className="flex rounded-lg overflow-hidden border border-white/10 text-sm">
          <button
            onClick={() => setScope("project")}
            className="flex-1 py-1.5 transition-colors"
            style={{
              background: scope === "project" ? "var(--accent)" : "transparent",
              color: scope === "project" ? "white" : "#c7c7d1",
            }}
            title={projectRoot}
          >
            {t.app.scopeProject}
          </button>
          <button
            onClick={() => setScope("global")}
            className="flex-1 py-1.5 transition-colors"
            style={{
              background: scope === "global" ? "var(--accent)" : "transparent",
              color: scope === "global" ? "white" : "#c7c7d1",
            }}
            title={globalRoot}
          >
            {t.app.scopeGlobal}
          </button>
        </div>
        <div className="text-[10px] opacity-40 mt-1.5 px-1 truncate" title={scope === "project" ? projectRoot : globalRoot}>
          {scope === "project" ? projectRoot : globalRoot}
        </div>
      </div>

      <nav className="flex-1 px-3 mt-4 space-y-1">
        {CATEGORIES.map((c) => {
          const active = category === c.id;
          const invalid = invalidCounts[c.id] ?? 0;
          return (
            <button
              key={c.id}
              disabled={disabled}
              onClick={() => setCategory(c.id)}
              className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors${disabled ? " opacity-40 cursor-not-allowed" : ""}`}
              style={{
                background: active ? "rgba(124,92,255,0.18)" : "transparent",
                color: active ? "#fff" : "#c7c7d1",
              }}
            >
              <span className="flex-1 text-left">{c.label}</span>
              {invalid > 0 && (
                <span
                  className="text-[10px] px-1.5 py-0.5 rounded-full"
                  style={{ background: "var(--danger)", color: "white" }}
                  title={t.app.invalidCountTitle(invalid)}
                >
                  {invalid}
                </span>
              )}
              {c.showCount !== false && <span className="text-xs opacity-40">{counts[c.id] ?? 0}</span>}
            </button>
          );
        })}
      </nav>

      <div className="px-4 py-3 text-[11px] opacity-40 border-t border-white/10">
        {t.app.footer}
      </div>
    </aside>
  );
}
