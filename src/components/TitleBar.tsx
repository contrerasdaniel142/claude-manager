import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { useLanguage } from "./LanguageProvider";
import { useToast } from "./ToastProvider";

const RECENTS_KEY = "clman-recent-projects";
const MAX_RECENTS = 8;

interface RecentProject {
  path: string;
  name: string;
}

function baseName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() || path;
}

function loadRecents(): RecentProject[] {
  try {
    const raw = localStorage.getItem(RECENTS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function pushRecent(path: string): RecentProject[] {
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
  onProjectChanged: (path: string) => void;
}

const win = getCurrentWindow();

export default function TitleBar({ projectPath, onProjectChanged }: Props) {
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    if (!menuOpen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setMenuOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [menuOpen]);
  const [recents, setRecents] = useState<RecentProject[]>(loadRecents);
  const toast = useToast();
  const { t } = useLanguage();

  const switchTo = async (path: string) => {
    setMenuOpen(false);
    try {
      await api.setProjectRoot(path);
      setRecents(pushRecent(path));
      onProjectChanged(path);
      toast.success(t.titleBar.switchedToast(baseName(path)));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const openFolder = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") await switchTo(picked);
  };

  return (
    <div
      data-tauri-drag-region
      className="h-9 flex items-center justify-between shrink-0 select-none relative"
      style={{ background: "var(--bg-sidebar)", color: "#c7c7d1" }}
    >
      <div className="flex items-center gap-2 px-3 h-full">
        <span className="text-xs font-medium opacity-70">{t.app.title}</span>
        <button
          onClick={() => setMenuOpen((v) => !v)}
          className="text-xs px-2 py-0.5 rounded hover:bg-white/10"
          title={projectPath}
        >
          {baseName(projectPath)} ▾
        </button>
      </div>

      {menuOpen && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setMenuOpen(false)} />
          <div
            className="absolute top-8 left-16 z-50 w-64 rounded-lg border shadow-lg text-xs overflow-hidden"
            style={{ background: "var(--bg-elevated)", borderColor: "var(--border)", color: "var(--text)" }}
          >
            <button className="w-full text-left px-3 py-2 hover:opacity-80" onClick={openFolder}>
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
                    onClick={() => switchTo(r.path)}
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
                  setMenuOpen(false);
                  toast.success(t.titleBar.noUpdates);
                }}
              >
                {t.titleBar.checkForUpdates}
              </button>
            </div>
          </div>
        </>
      )}

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
