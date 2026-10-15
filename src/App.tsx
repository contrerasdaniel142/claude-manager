import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { api } from "./api";
import Sidebar from "./components/Sidebar";
import AdvisorView from "./components/AdvisorView";
import AgentsView from "./components/AgentsView";
import SkillsView from "./components/SkillsView";
import HooksView from "./components/HooksView";
import ClaudeMdView from "./components/ClaudeMdView";
import CommandPalette, { type PaletteCommand } from "./components/CommandPalette";
import GlobalSearch from "./components/GlobalSearch";
import McpServersView from "./components/McpServersView";
import OutputStylesView from "./components/OutputStylesView";
import PermissionsView from "./components/PermissionsView";
import PluginsView from "./components/PluginsView";
import RulesView from "./components/RulesView";
import LanguageProvider, { useLanguage } from "./components/LanguageProvider";
import StatusBar, { type ProblemItem } from "./components/StatusBar";
import TitleBar, { getLastProject, loadRecents, pushRecent, removeRecent, baseName, type RecentProject } from "./components/TitleBar";
import ToastProvider, { useToast } from "./components/ToastProvider";
import WelcomeView from "./components/WelcomeView";
import { open } from "@tauri-apps/plugin-dialog";
import type { Category, Scope } from "./types";

export default function App() {
  return (
    <LanguageProvider>
      <ToastProvider>
        <AppContent />
      </ToastProvider>
    </LanguageProvider>
  );
}

// Keyboard-shortcut order (Ctrl+1..9) — mirrors Sidebar's own category list;
// keep both in sync if either changes.
const CATEGORY_ORDER: Category[] = [
  "advisor",
  "steering",
  "rules",
  "hooks",
  "agents",
  "skills",
  "outputStyles",
  "permissions",
  "mcpServers",
  "plugins",
];

function AppContent() {
  const { t } = useLanguage();
  const toast = useToast();
  const updateChecked = useRef(false);
  const restoreAttempted = useRef(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [openTarget, setOpenTarget] = useState<{ category: Category; id: string } | null>(null);
  const [scope, setScope] = useState<Scope>("project");
  const [category, setCategory] = useState<Category>("steering");
  const [roots, setRoots] = useState({ project: "", global: "" });
  const [recents, setRecents] = useState<RecentProject[]>(loadRecents);

  const switchTo = async (path: string) => {
    try {
      await api.setProjectRoot(path);
      setRecents(pushRecent(path));
      setRoots((r) => ({ ...r, project: path }));
      setRefreshSignal((n) => n + 1);
      toast.success(t.titleBar.switchedToast(baseName(path)));
    } catch (e) {
      toast.error(String(e));
    }
  };
  const openFolder = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") await switchTo(picked);
  };
  const [counts, setCounts] = useState<Record<Category, number>>({
    advisor: 0,
    steering: 0,
    rules: 0,
    hooks: 0,
    agents: 0,
    skills: 0,
    outputStyles: 0,
    permissions: 0,
    mcpServers: 0,
    plugins: 0,
  });
  const [invalidCounts, setInvalidCounts] = useState<Record<Category, number>>({
    advisor: 0,
    steering: 0,
    rules: 0,
    hooks: 0,
    agents: 0,
    skills: 0,
    outputStyles: 0,
    permissions: 0,
    mcpServers: 0,
    plugins: 0,
  });
  const [refreshSignal, setRefreshSignal] = useState(0);
  const [prefill, setPrefill] = useState<{ category: Category; data: Record<string, unknown> } | null>(null);
  const [problemSources, setProblemSources] = useState<{
    agents: ProblemItem[];
    skills: ProblemItem[];
    hooks: ProblemItem[];
  }>({ agents: [], skills: [], hooks: [] });
  const problems = useMemo(
    () => [...problemSources.agents, ...problemSources.skills, ...problemSources.hooks],
    [problemSources],
  );

  // Advisor sends the user here with a specific category and (usually) the
  // fields an AI call already generated for it — each target view applies
  // them as if its own AiAssistBox had just returned that same result.
  const navigate = (targetCategory: Category, data: Record<string, unknown> | null) => {
    setCategory(targetCategory);
    if (data) setPrefill({ category: targetCategory, data });
  };

  const prefillFor = (targetCategory: Category) =>
    prefill?.category === targetCategory ? prefill.data : undefined;
  const clearPrefill = () => setPrefill(null);

  // Global Search sends the user to an existing item by id — "hooks" has no
  // stable per-rule id (array index only), so it just switches category.
  const openSearchResult = (targetCategory: Category, id: string) => {
    setCategory(targetCategory);
    if (targetCategory !== "hooks") setOpenTarget({ category: targetCategory, id });
  };
  const openIdFor = (targetCategory: Category) =>
    openTarget?.category === targetCategory ? openTarget.id : undefined;
  const clearOpenTarget = () => setOpenTarget(null);

  // Ctrl+K toggles the command palette, Ctrl+Shift+F toggles global search,
  // Ctrl+1..9 jumps straight to a sidebar category without opening it — all
  // ignore typing in a field since they require the Ctrl modifier, which
  // plain text input never sends.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "f") {
        e.preventDefault();
        setSearchOpen((v) => !v);
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen((v) => !v);
      } else if ((e.ctrlKey || e.metaKey) && /^[1-9]$/.test(e.key)) {
        const idx = Number(e.key) - 1;
        if (idx < CATEGORY_ORDER.length) {
          e.preventDefault();
          setCategory(CATEGORY_ORDER[idx]);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const paletteCommands = useMemo<PaletteCommand[]>(() => {
    const navCommands: PaletteCommand[] = CATEGORY_ORDER.map((cat, i) => ({
      id: `go-${cat}`,
      label: `${t.commandPalette.goTo} ${t.app.categories[cat]}`,
      hint: `Ctrl+${i + 1}`,
      action: () => setCategory(cat),
    }));
    return [
      ...navCommands,
      { id: "scope-project", label: t.commandPalette.switchToProject, action: () => setScope("project") },
      { id: "scope-global", label: t.commandPalette.switchToGlobal, action: () => setScope("global") },
      {
        id: "global-search",
        label: t.commandPalette.openSearch,
        hint: "Ctrl+Shift+F",
        action: () => setSearchOpen(true),
      },
    ];
  }, [t]);

  const runUpdateCheck = async (opts: { silent: boolean }) => {
    try {
      const update = await check();
      if (!update) {
        if (!opts.silent) {
          toast.success(t.titleBar.noUpdates);
        }
        return;
      }

      let installing = false;
      let toastId: number | null = null;
      const msg = t.updater.available.replace("{version}", update.version);

      const showUpdateToast = () => {
        const options = {
          sticky: true,
          action: {
            label: installing ? t.updater.installing : t.updater.install,
            disabled: installing,
            onClick: async () => {
              if (installing) return;
              installing = true;
              showUpdateToast();
              try {
                await update.downloadAndInstall();
                await relaunch();
              } catch (err) {
                installing = false;
                showUpdateToast();
                toast.error(String(err));
              }
            },
          },
        };

        if (toastId === null) {
          toastId = toast.info(msg, options);
        } else {
          toast.update(toastId, msg, options);
        }
      };

      showUpdateToast();
    } catch (err) {
      if (opts.silent) {
        console.warn("Update check failed:", err);
      } else {
        toast.error(String(err));
      }
    }
  };

  useEffect(() => {
    api.getRoots().then((r) => {
      setRoots(r);
      if (restoreAttempted.current) return;
      restoreAttempted.current = true;
      const last = getLastProject();
      if (!last || last === r.project) return;
      api
        .setProjectRoot(last)
        .then(() => {
          setRoots((prev) => ({ ...prev, project: last }));
          setRefreshSignal((n) => n + 1);
        })
        .catch(() => {
          // Folder was moved or deleted since last session — drop the stale recent and tell the user.
          setRecents(removeRecent(last));
          toast.error(t.titleBar.restoreFailedToast(baseName(last)));
        });
    });
  }, []);

  useEffect(() => {
    if (updateChecked.current) return;
    updateChecked.current = true;
    runUpdateCheck({ silent: true });
  }, []);

  // Un archivo tocado por fuera de la app (editado a mano, git pull, otra
  // instancia de clman) dispara este evento desde el backend — ver
  // src-tauri/src/watcher.rs. Bumpeamos un contador en vez de pasar el
  // evento en sí porque lo único que le importa a cada vista es "algo
  // cambió", no qué cambió puntualmente.
  useEffect(() => {
    const unlisten = listen("claude-files-changed", () => {
      setRefreshSignal((n) => n + 1);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // The sidebar counts/badges need real numbers before the user ever opens
  // a given module — each view only reports its own count on mount via
  // onListChanged, so without this a module you haven't clicked into yet
  // shows 0 even when it already has things saved.
  useEffect(() => {
    let cancelled = false;
    Promise.all([
      api.listSteering(scope),
      api.listRules(scope),
      api.readHooks(scope),
      api.listAgents(scope),
      api.listSkills(scope),
      api.listOutputStyles(scope),
      api.readPermissions(scope),
      api.listMcpServers(scope),
      api.listInstalledPlugins(),
    ]).then(
      ([
        steeringList,
        rulesList,
        hooksDoc,
        agentsList,
        skillsList,
        outputStylesList,
        permissions,
        mcpServersList,
        installedPlugins,
      ]) => {
        if (cancelled) return;
        setCounts({
          advisor: 0,
          steering: steeringList.length,
          rules: rulesList.length,
          hooks: hooksDoc.rules.length,
          agents: agentsList.length,
          skills: skillsList.length,
          outputStyles: outputStylesList.length,
          permissions: permissions.allow.length + permissions.ask.length + permissions.deny.length,
          mcpServers: mcpServersList.length,
          plugins: installedPlugins.length,
        });
        setInvalidCounts((c) => ({
          ...c,
          agents: agentsList.filter((a) => !a.valid).length,
          skills: skillsList.filter((s) => !s.valid).length,
        }));
      },
    );
    return () => {
      cancelled = true;
    };
  }, [scope, refreshSignal]);

  const hasOpenedProject = recents.length > 0;

  return (
    <div className="flex flex-col h-screen overflow-hidden text-sm">
      <CommandPalette
        open={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        commands={paletteCommands}
        placeholder={t.commandPalette.placeholder}
      />
      <GlobalSearch
        open={searchOpen}
        scope={scope}
        onClose={() => setSearchOpen(false)}
        onOpenResult={openSearchResult}
      />
      <TitleBar
        projectPath={roots.project}
        hasOpenedProject={hasOpenedProject}
        recents={recents}
        onOpenFolder={openFolder}
        onSwitchTo={switchTo}
        onCheckForUpdates={() => runUpdateCheck({ silent: false })}
      />
    <div className="flex flex-1 overflow-hidden">
      <Sidebar
        scope={scope}
        setScope={setScope}
        category={category}
        setCategory={setCategory}
        counts={counts}
        invalidCounts={invalidCounts}
        projectRoot={roots.project}
        globalRoot={roots.global}
      />
      <main className="flex-1 overflow-hidden" style={{ background: "var(--bg)" }}>
        {!hasOpenedProject && scope === "project" ? (
          <WelcomeView recents={recents} onOpenFolder={openFolder} onSwitchTo={switchTo} />
        ) : (
          <>
            {category === "advisor" && <AdvisorView onNavigate={navigate} />}
            {category === "steering" && (
              <ClaudeMdView
                scope={scope}
                refreshSignal={refreshSignal}
                onListChanged={(list) => setCounts((c) => ({ ...c, steering: list.length }))}
              />
            )}
            {category === "rules" && (
              <RulesView
                scope={scope}
                refreshSignal={refreshSignal}
                onListChanged={(list) => setCounts((c) => ({ ...c, rules: list.length }))}
                prefill={prefillFor("rules")}
                onPrefillConsumed={clearPrefill}
                openId={openIdFor("rules")}
                onOpenIdConsumed={clearOpenTarget}
              />
            )}
            {category === "hooks" && (
              <HooksView
                scope={scope}
                refreshSignal={refreshSignal}
                onCountChanged={(n) => setCounts((c) => ({ ...c, hooks: n }))}
                onLoadErrorsChanged={(errors) =>
                  setProblemSources((p) => ({
                    ...p,
                    hooks: errors.map((message) => ({ category: "hooks" as Category, title: t.hooks.settingsJsonErrors, message })),
                  }))
                }
                prefill={prefillFor("hooks")}
                onPrefillConsumed={clearPrefill}
              />
            )}
            {category === "agents" && (
              <AgentsView
                scope={scope}
                refreshSignal={refreshSignal}
                onListChanged={(list) => {
                  setCounts((c) => ({ ...c, agents: list.length }));
                  const invalid = list.filter((a) => !a.valid);
                  setInvalidCounts((c) => ({ ...c, agents: invalid.length }));
                  setProblemSources((p) => ({
                    ...p,
                    agents: invalid.map((a) => ({
                      category: "agents" as Category,
                      title: a.name || a.id,
                      message: a.errors.join(" "),
                    })),
                  }));
                }}
                openId={openIdFor("agents")}
                onOpenIdConsumed={clearOpenTarget}
              />
            )}
            {category === "skills" && (
              <SkillsView
                scope={scope}
                refreshSignal={refreshSignal}
                onListChanged={(list) => {
                  setCounts((c) => ({ ...c, skills: list.length }));
                  const invalid = list.filter((s) => !s.valid);
                  setInvalidCounts((c) => ({ ...c, skills: invalid.length }));
                  setProblemSources((p) => ({
                    ...p,
                    skills: invalid.map((s) => ({
                      category: "skills" as Category,
                      title: s.name || s.id,
                      message: s.errors.join(" "),
                    })),
                  }));
                }}
                prefill={prefillFor("skills")}
                onPrefillConsumed={clearPrefill}
                openId={openIdFor("skills")}
                onOpenIdConsumed={clearOpenTarget}
              />
            )}
            {category === "outputStyles" && (
              <OutputStylesView
                scope={scope}
                refreshSignal={refreshSignal}
                onListChanged={(list) => setCounts((c) => ({ ...c, outputStyles: list.length }))}
                prefill={prefillFor("outputStyles")}
                onPrefillConsumed={clearPrefill}
                openId={openIdFor("outputStyles")}
                onOpenIdConsumed={clearOpenTarget}
              />
            )}
            {category === "permissions" && (
              <PermissionsView
                scope={scope}
                refreshSignal={refreshSignal}
                onCountChanged={(n) => setCounts((c) => ({ ...c, permissions: n }))}
                prefill={prefillFor("permissions")}
                onPrefillConsumed={clearPrefill}
              />
            )}
            {category === "mcpServers" && (
              <McpServersView
                scope={scope}
                refreshSignal={refreshSignal}
                onListChanged={(list) => setCounts((c) => ({ ...c, mcpServers: list.length }))}
                prefill={prefillFor("mcpServers")}
                onPrefillConsumed={clearPrefill}
                openId={openIdFor("mcpServers")}
                onOpenIdConsumed={clearOpenTarget}
              />
            )}
            {category === "plugins" && <PluginsView scope={scope} />}
          </>
        )}
      </main>
    </div>
    <StatusBar scope={scope} problems={problems} onNavigate={setCategory} />
    </div>
  );
}
