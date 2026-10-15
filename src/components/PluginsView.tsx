import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { MarketplaceSummary, PluginSummary, Scope } from "../types";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import Select from "./Select";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";

interface Props {
  scope: Scope;
}

const toCliScope = (scope: Scope) => (scope === "project" ? "project" : "user");

// One page at a time — the real catalog is thousands of entries, so neither
// the backend nor this component ever holds more than what's actually shown.
const PAGE_SIZE = 50;

export default function PluginsView({ scope }: Props) {
  const [marketplaces, setMarketplaces] = useState<MarketplaceSummary[]>([]);
  const [installed, setInstalled] = useState<PluginSummary[]>([]);
  const [availableItems, setAvailableItems] = useState<PluginSummary[]>([]);
  const [availableTotal, setAvailableTotal] = useState(0);
  const [availableLoading, setAvailableLoading] = useState(false);
  const [newSource, setNewSource] = useState("");
  const [marketplaceScope, setMarketplaceScope] = useState(toCliScope(scope));
  const [installScope, setInstallScope] = useState(toCliScope(scope));
  const [availableMarketplace, setAvailableMarketplace] = useState("");
  const [availableSearch, setAvailableSearch] = useState("");
  const [busy, setBusy] = useState<Set<string>>(new Set());
  const [pendingConfirm, setPendingConfirm] = useState<{
    id: string;
    name: string;
    marketplace: string;
    scope: string;
    command: string;
    sha256: string;
  } | null>(null);
  const toast = useToast();
  const { t } = useLanguage();
  const searchRequestId = useRef(0);

  const refresh = async () => {
    const [mp, inst] = await Promise.all([api.listMarketplaces(), api.listInstalledPlugins()]);
    setMarketplaces(mp);
    setInstalled(inst);
    setAvailableMarketplace((current) => (current && mp.some((m) => m.name === current) ? current : mp[0]?.name ?? ""));
  };

  useEffect(() => {
    refresh();
    setMarketplaceScope(toCliScope(scope));
    setInstallScope(toCliScope(scope));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  const runAvailableSearch = async (offset: number, append: boolean, force = false) => {
    if (!availableMarketplace) {
      setAvailableItems([]);
      setAvailableTotal(0);
      return;
    }
    const requestId = ++searchRequestId.current;
    setAvailableLoading(true);
    try {
      const page = await api.searchAvailablePlugins(
        availableMarketplace,
        availableSearch.trim(),
        offset,
        PAGE_SIZE,
        force,
      );
      if (requestId !== searchRequestId.current) return; // a newer search landed first
      setAvailableItems((prev) => (append ? [...prev, ...page.items] : page.items));
      setAvailableTotal(page.total);
    } catch (e) {
      if (requestId === searchRequestId.current) toast.error(String(e));
    } finally {
      if (requestId === searchRequestId.current) setAvailableLoading(false);
    }
  };

  // Debounced so typing doesn't fire a search (and a cold `claude plugin
  // list --json --available` parse, the first time) per keystroke.
  useEffect(() => {
    const timer = setTimeout(() => runAvailableSearch(0, false), 200);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [availableMarketplace, availableSearch]);

  const withBusy = async (key: string, fn: () => Promise<string | void>) => {
    setBusy((prev) => new Set(prev).add(key));
    try {
      const message = await fn();
      if (message) {
        toast.success(message);
      }
      await refresh();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  const handleRefreshAll = async () => {
    setBusy((prev) => new Set(prev).add("refresh-all"));
    try {
      await refresh();
      await runAvailableSearch(0, false, true);
      toast.success(t.plugins.refreshAllSuccess);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy((prev) => {
        const next = new Set(prev);
        next.delete("refresh-all");
        return next;
      });
    }
  };

  const addMarketplace = () => {
    if (!newSource.trim()) return;
    withBusy(`add-mp-${newSource}`, async () => {
      const msg = await api.addMarketplace(newSource.trim(), marketplaceScope);
      setNewSource("");
      await runAvailableSearch(0, false); // marketplace catalog may have changed
      return msg;
    });
  };

  return (
    <div className="h-full overflow-y-auto p-5">
      <div className="max-w-2xl space-y-6">
        <div className="flex items-center justify-end">
          <Button
            size="sm"
            variant="ghost"
            disabled={busy.has("refresh-all")}
            onClick={handleRefreshAll}
          >
            {t.plugins.refreshAll}
          </Button>
        </div>

        <p className="text-xs" style={{ color: "var(--text-muted)" }}>
          {t.plugins.intro}
        </p>

        <div>
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold">{t.plugins.marketplacesTitle}</h2>
          </div>
          <div className="flex gap-1 mt-2">
            <TextInput
              value={newSource}
              onChange={(e) => setNewSource(e.target.value)}
              placeholder={t.plugins.marketplaceSourcePlaceholder}
              className="flex-1"
              onKeyDown={(e) => e.key === "Enter" && addMarketplace()}
            />
            <Select value={marketplaceScope} onChange={(e) => setMarketplaceScope(e.target.value)} className="w-28">
              <option value="project">project</option>
              <option value="user">user</option>
            </Select>
            <Button onClick={addMarketplace} disabled={busy.has(`add-mp-${newSource}`)}>
              {t.plugins.addMarketplace}
            </Button>
          </div>

          <div className="mt-2 space-y-1.5">
            {marketplaces.length === 0 && (
              <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                {t.plugins.noMarketplaces}
              </p>
            )}
            {marketplaces.map((m) => (
              <div
                key={m.name}
                className="p-2.5 rounded-lg border flex items-start gap-3"
                style={{ borderColor: "var(--border)" }}
              >
                <div className="min-w-0 flex-1">
                  <div className="text-sm font-medium truncate">{m.name}</div>
                  <div className="text-xs mt-0.5 truncate" style={{ color: "var(--text-muted)" }}>
                    {m.source} — {m.repo || m.url || m.path || ""}
                  </div>
                </div>
                <Button
                  size="sm"
                  variant="danger"
                  className="shrink-0"
                  disabled={busy.has(`rm-mp-${m.name}`)}
                  onClick={() =>
                    withBusy(`rm-mp-${m.name}`, async () => {
                      const msg = await api.removeMarketplace(m.name);
                      await runAvailableSearch(0, false); // marketplace catalog may have changed
                      return msg;
                    })
                  }
                >
                  {t.plugins.remove}
                </Button>
              </div>
            ))}
          </div>
        </div>

        <div>
          <h2 className="text-sm font-semibold">{t.plugins.pluginsTitle}</h2>

          {installed.length === 0 && marketplaces.length === 0 && (
            <p className="text-xs mt-2" style={{ color: "var(--text-muted)" }}>
              {t.plugins.noPlugins}
            </p>
          )}

          {installed.length > 0 && (
            <div className="mt-2">
              <div className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.plugins.installedSection}
              </div>
              <div className="mt-1 space-y-1.5">
                {installed.map((p) => (
                  <div
                    key={p.id}
                    className="p-2.5 rounded-lg border flex items-start gap-2"
                    style={{ borderColor: "var(--border)" }}
                  >
                    <div className="min-w-0 flex-1">
                      <div className="text-sm font-medium truncate">{p.id}</div>
                      <div className="text-xs mt-0.5" style={{ color: "var(--text-muted)" }}>
                        v{p.version} — scope: {p.scope} — {p.enabled ? t.plugins.enable : t.plugins.disable}
                      </div>
                    </div>
                    <Button
                      size="sm"
                      variant={p.enabled ? "ghost" : "primary"}
                      disabled={busy.has(`toggle-${p.id}`)}
                      onClick={() =>
                        withBusy(`toggle-${p.id}`, () =>
                          p.enabled
                            ? api.disablePlugin(p.name, p.marketplace, p.scope)
                            : api.enablePlugin(p.name, p.marketplace, p.scope),
                        )
                      }
                    >
                      {p.enabled ? t.plugins.disable : t.plugins.enable}
                    </Button>
                    <Button
                      size="sm"
                      variant="danger"
                      disabled={busy.has(`uninstall-${p.id}`)}
                      onClick={() =>
                        withBusy(`uninstall-${p.id}`, () => api.uninstallPlugin(p.name, p.marketplace, p.scope))
                      }
                    >
                      {t.plugins.uninstall}
                    </Button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {marketplaces.length > 0 && (
            <div className="mt-4">
              <div className="flex items-center justify-between gap-2">
                <div className="text-xs font-medium shrink-0" style={{ color: "var(--text-muted)" }}>
                  {t.plugins.availableSection}
                </div>
                <Select
                  value={availableMarketplace}
                  onChange={(e) => setAvailableMarketplace(e.target.value)}
                  compact
                >
                  {marketplaces.map((m) => (
                    <option key={m.name} value={m.name}>
                      {m.name}
                    </option>
                  ))}
                </Select>
                <Select value={installScope} onChange={(e) => setInstallScope(e.target.value)} compact>
                  <option value="project">project</option>
                  <option value="user">user</option>
                  <option value="local">local</option>
                </Select>
              </div>
              <TextInput
                value={availableSearch}
                onChange={(e) => setAvailableSearch(e.target.value)}
                placeholder={t.plugins.searchPlaceholder}
                className="w-full mt-1.5"
                small
              />
              {availableTotal > 0 && (
                <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                  {t.plugins.showingCount(availableItems.length, availableTotal)}
                </div>
              )}
              <div className="mt-1.5 space-y-1.5">
                {availableItems.map((p) => {
                  const isPending = pendingConfirm?.id === p.id;
                  return (
                    <div
                      key={p.id}
                      className="p-2.5 rounded-lg border space-y-2.5"
                      style={{ borderColor: "var(--border)" }}
                    >
                      <div className="flex items-start gap-3">
                        <div className="min-w-0 flex-1">
                          <div className="text-sm font-medium truncate">{p.id}</div>
                          {p.description && (
                            <div className="text-xs mt-0.5" style={{ color: "var(--text-muted)" }}>
                              {p.description}
                            </div>
                          )}
                        </div>
                        <Button
                          size="sm"
                          className="shrink-0"
                          disabled={busy.has(`install-${p.id}`)}
                          onClick={() =>
                            withBusy(`install-${p.id}`, async () => {
                              const res = await api.installPlugin(p.name, p.marketplace, installScope);
                              if (res.status === "needs_confirmation") {
                                setPendingConfirm({
                                  id: p.id,
                                  name: p.name,
                                  marketplace: p.marketplace,
                                  scope: installScope,
                                  command: res.command ?? "",
                                  sha256: res.sha256 ?? "",
                                });
                                return;
                              }
                              setPendingConfirm(null);
                              return res.message;
                            })
                          }
                        >
                          {t.plugins.install}
                        </Button>
                      </div>

                      {isPending && (
                        <div
                          className="p-2.5 rounded border space-y-2 text-xs"
                          style={{
                            borderColor: "var(--border)",
                            backgroundColor: "var(--bg-subtle, rgba(255, 255, 255, 0.03))",
                          }}
                        >
                          <p style={{ color: "var(--text-muted)" }}>{t.plugins.confirmCommandWarning}</p>
                          <pre
                            className="p-2 rounded font-mono text-xs overflow-x-auto whitespace-pre-wrap break-all select-text"
                            style={{
                              backgroundColor: "var(--bg-code, rgba(0, 0, 0, 0.2))",
                              border: "1px solid var(--border)",
                            }}
                          >
                            <code>{pendingConfirm.command}</code>
                          </pre>
                          <div className="flex items-center gap-2">
                            <Button
                              size="sm"
                              disabled={busy.has(`install-${p.id}`)}
                              onClick={() =>
                                withBusy(`install-${p.id}`, async () => {
                                  const res = await api.confirmPluginInstall(
                                    pendingConfirm.name,
                                    pendingConfirm.marketplace,
                                    pendingConfirm.scope,
                                    pendingConfirm.sha256,
                                  );
                                  if (res.status === "needs_confirmation") {
                                    setPendingConfirm({
                                      id: p.id,
                                      name: pendingConfirm.name,
                                      marketplace: pendingConfirm.marketplace,
                                      scope: pendingConfirm.scope,
                                      command: res.command ?? "",
                                      sha256: res.sha256 ?? "",
                                    });
                                    return;
                                  }
                                  setPendingConfirm(null);
                                  return res.message;
                                })
                              }
                            >
                              {t.plugins.confirmCommandApprove}
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              disabled={busy.has(`install-${p.id}`)}
                              onClick={() => setPendingConfirm(null)}
                            >
                              {t.plugins.confirmCommandCancel}
                            </Button>
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
                {availableItems.length === 0 && !availableLoading && (
                  <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                    {t.plugins.noneFound}
                  </p>
                )}
                {availableLoading && (
                  <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                    {t.plugins.searching}
                  </p>
                )}
                {!availableLoading && availableItems.length < availableTotal && (
                  <Button size="sm" variant="ghost" onClick={() => runAvailableSearch(availableItems.length, true)}>
                    {t.plugins.loadMore}
                  </Button>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
