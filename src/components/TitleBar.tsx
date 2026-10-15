import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useLanguage } from "./LanguageProvider";

const RECENTS_KEY = "clman-recent-projects";
const MAX_RECENTS = 8;

export interface RecentProject {
  path: string;
  name: string;
}

export function baseName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() || path;
}

export function loadRecents(): RecentProject[] {
  try {
    const raw = localStorage.getItem(RECENTS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

export function getLastProject(): string | null {
  return loadRecents()[0]?.path ?? null;
}

export function pushRecent(path: string): RecentProject[] {
  const updated = [{ path, name: baseName(path) }, ...loadRecents().filter((r) => r.path !== path)].slice(
    0,
    MAX_RECENTS,
  );
  try {
    localStorage.setItem(RECENTS_KEY, JSON.stringify(updated));
  } catch {
    // per-viewer convenience only — if it can't save, the menu just won't remember it.
  }
  return updated;
}

interface Props {
  projectPath: string;
  hasOpenedProject: boolean;
  recents: RecentProject[];
  onOpenFolder: () => void;
  onSwitchTo: (path: string) => void;
  onCheckForUpdates: () => void;
}

const win = getCurrentWindow();

export default function TitleBar({
  projectPath,
  hasOpenedProject,
  recents,
  onOpenFolder,
  onSwitchTo,
  onCheckForUpdates,
}: Props) {
  const [openMenu, setOpenMenu] = useState<"project" | "language" | null>(null);

  useEffect(() => {
    if (!openMenu) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpenMenu(null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [openMenu]);
  const { lang, setLang, t } = useLanguage();

  return (
    <div
      data-tauri-drag-region
      className="h-9 flex items-center justify-between shrink-0 select-none relative"
      style={{ background: "var(--bg-sidebar)", color: "#c7c7d1" }}
    >
      <div className="flex items-center gap-2 px-3 h-full">
        <span className="text-xs font-medium opacity-70">{t.app.title}</span>
        <div className="relative">
          <button
            onClick={() => setOpenMenu((v) => (v === "language" ? null : "language"))}
            className="text-xs px-2 py-0.5 rounded hover:bg-white/10"
            title={t.app.language}
          >
            {lang.toUpperCase()} ▾
          </button>
          {openMenu === "language" && (
            <>
              <div className="fixed inset-0 z-40" onClick={() => setOpenMenu(null)} />
              <div
                className="absolute top-8 left-0 z-50 w-24 rounded-lg border shadow-lg text-xs overflow-hidden"
                style={{ background: "var(--bg-elevated)", borderColor: "var(--border)", color: "var(--text)" }}
              >
                <button className="w-full text-left px-3 py-2 hover:opacity-80" onClick={() => { setOpenMenu(null); setLang("en"); }}>EN</button>
                <button className="w-full text-left px-3 py-2 hover:opacity-80" onClick={() => { setOpenMenu(null); setLang("es"); }}>ES</button>
              </div>
            </>
          )}
        </div>
        <div className="relative">
          <button
            onClick={() => setOpenMenu((v) => (v === "project" ? null : "project"))}
            className="text-xs px-2 py-0.5 rounded hover:bg-white/10"
            title={hasOpenedProject ? projectPath : undefined}
          >
            {hasOpenedProject ? baseName(projectPath) : t.titleBar.noProjectOpen} ▾
          </button>
          {openMenu === "project" && (
            <>
              <div className="fixed inset-0 z-40" onClick={() => setOpenMenu(null)} />
              <div
                className="absolute top-8 left-0 z-50 w-64 rounded-lg border shadow-lg text-xs overflow-hidden"
                style={{ background: "var(--bg-elevated)", borderColor: "var(--border)", color: "var(--text)" }}
              >
                <button
                  className="w-full text-left px-3 py-2 hover:opacity-80"
                  onClick={() => {
                    setOpenMenu(null);
                    onOpenFolder();
                  }}
                >
                  {t.titleBar.openFolder}
                </button>
                {recents.length > 0 && (
                  <div className="border-t" style={{ borderColor: "var(--border)" }}>
                    <div className="px-3 py-1 opacity-50">{t.titleBar.recent}</div>
                    {recents.map((r) => (
                      <button
                        key={r.path}
                        className="w-full text-left px-3 py-1.5 hover:opacity-80 truncate block"
                        title={r.path}
                        onClick={() => {
                          setOpenMenu(null);
                          onSwitchTo(r.path);
                        }}
                      >
                        {r.name}
                      </button>
                    ))}
                  </div>
                )}
                <div className="border-t" style={{ borderColor: "var(--border)" }}>
                  <button
                    className="w-full text-left px-3 py-2 hover:opacity-80"
                    onClick={() => {
                      setOpenMenu(null);
                      onCheckForUpdates();
                    }}
                  >
                    {t.titleBar.checkForUpdates}
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      </div>

      <div className="flex items-center h-full">
        <button
          className="w-11 h-full flex items-center justify-center hover:bg-white/10"
          onClick={() => win.minimize()}
        >
          &#x2013;
        </button>
        <button
          className="w-11 h-full flex items-center justify-center hover:bg-white/10"
          onClick={() => win.toggleMaximize()}
        >
          &#x25A1;
        </button>
        <button
          className="w-11 h-full flex items-center justify-center hover:bg-[#e81123] hover:text-white"
          onClick={() => win.close()}
        >
          &#x2715;
        </button>
      </div>
    </div>
  );
}
