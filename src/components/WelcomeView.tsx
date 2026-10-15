import type { RecentProject } from "./TitleBar";
import { useLanguage } from "./LanguageProvider";
import Button from "./Button";

interface Props {
  recents: RecentProject[];
  onOpenFolder: () => void;
  onSwitchTo: (path: string) => void;
}

export default function WelcomeView({ recents, onOpenFolder, onSwitchTo }: Props) {
  const { t } = useLanguage();

  return (
    <div
      className="h-full flex flex-col items-center justify-center p-8 select-none"
      style={{ background: "var(--bg)", color: "var(--text)" }}
    >
      <div className="max-w-md w-full flex flex-col items-center text-center">
        <h1 className="text-2xl font-semibold mb-2">{t.app.title}</h1>
        <p className="text-sm mb-6" style={{ color: "var(--text-muted)" }}>
          {t.welcome.description}
        </p>

        <Button variant="primary" onClick={onOpenFolder}>
          {t.welcome.openFolder}
        </Button>

        {recents.length > 0 && (
          <div className="w-full mt-10 text-left">
            <h2
              className="text-xs font-semibold uppercase tracking-wider mb-3 px-1"
              style={{ color: "var(--text-muted)" }}
            >
              {t.welcome.recentProjects}
            </h2>
            <div
              className="flex flex-col rounded-lg border overflow-hidden"
              style={{ borderColor: "var(--border)" }}
            >
              {recents.map((r) => (
                <button
                  key={r.path}
                  title={r.path}
                  onClick={() => onSwitchTo(r.path)}
                  className="w-full text-left px-3 py-2.5 border-b last:border-b-0 hover:opacity-80 transition-opacity"
                  style={{ borderColor: "var(--border)" }}
                >
                  <div className="text-sm font-medium truncate" style={{ color: "var(--text)" }}>
                    {r.name}
                  </div>
                  <div className="text-xs truncate mt-0.5" style={{ color: "var(--text-muted)" }}>
                    {r.path}
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
