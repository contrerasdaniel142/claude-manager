import { useState } from "react";
import Button from "./Button";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";

/** Same shape as t.importSkill / t.importMcpServer — any "import X from a
 * repo" feature's strings fit this without a new dictionary key. */
export interface ImportBoxStrings {
  intro: string;
  urlLabel: string;
  urlPlaceholder: string;
  searching: string;
  search: string;
  noneFound: string;
  alreadyInstalled: string;
  installing: string;
  install: string;
}

interface Props<T> {
  strings: ImportBoxStrings;
  /** Names already installed locally — used to grey out a match instead of
   * letting it collide on import. */
  localNames: string[];
  search: (repoUrl: string) => Promise<T[]>;
  /** Return value is ignored — only whether it throws matters here. */
  install: (repoUrl: string, item: T) => Promise<unknown>;
  getKey: (item: T) => string;
  getName: (item: T) => string;
  getTitle: (item: T) => string;
  getSubtitle: (item: T) => string;
  onImported: () => void;
}

/** Generic "paste a repo URL, search it, install what it finds" box — the
 * search/list/install UI shared by every "import from a repo" feature
 * (skills, MCP servers, ...); only what to search for and how to install it
 * differ, so those come in as props instead of duplicating this shell. */
export default function ImportFromRepoBox<T>({
  strings,
  localNames,
  search: searchFn,
  install: installFn,
  getKey,
  getName,
  getTitle,
  getSubtitle,
  onImported,
}: Props<T>) {
  const [repoUrl, setRepoUrl] = useState("");
  const [found, setFound] = useState<T[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [importing, setImporting] = useState<Set<string>>(new Set());
  const toast = useToast();

  const search = async () => {
    if (!repoUrl.trim()) return;
    setLoading(true);
    setFound(null);
    try {
      setFound(await searchFn(repoUrl.trim()));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
    }
  };

  const install = async (item: T) => {
    const key = getKey(item);
    setImporting((prev) => new Set(prev).add(key));
    try {
      await installFn(repoUrl.trim(), item);
      onImported();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setImporting((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  return (
    <div className="max-w-2xl space-y-3">
      <p style={{ color: "var(--text-muted)" }}>{strings.intro}</p>
      <div>
        <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
          {strings.urlLabel}
        </label>
        <div className="flex gap-1 mt-1">
          <TextInput
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
            placeholder={strings.urlPlaceholder}
            className="flex-1"
            onKeyDown={(e) => e.key === "Enter" && search()}
          />
          <Button onClick={search} disabled={loading || !repoUrl.trim()} style={{ opacity: loading ? 0.6 : 1 }}>
            {loading ? strings.searching : strings.search}
          </Button>
        </div>
      </div>

      {found && found.length === 0 && <p style={{ color: "var(--text-muted)" }}>{strings.noneFound}</p>}

      {found && found.length > 0 && (
        <div className="space-y-1.5">
          {found.map((item) => {
            const key = getKey(item);
            const alreadyInstalled = localNames.includes(getName(item));
            return (
              <div
                key={key}
                className="p-2.5 rounded-lg border flex items-start gap-3"
                style={{ borderColor: "var(--border)" }}
              >
                <div className="min-w-0 flex-1">
                  <div className="text-sm font-medium truncate">{getTitle(item)}</div>
                  <div className="text-xs mt-0.5" style={{ color: "var(--text-muted)" }}>
                    {getSubtitle(item)}
                  </div>
                </div>
                {alreadyInstalled ? (
                  <span className="shrink-0 text-xs pt-0.5" style={{ color: "var(--text-muted)" }}>
                    {strings.alreadyInstalled}
                  </span>
                ) : (
                  <Button
                    size="sm"
                    className="shrink-0"
                    onClick={() => install(item)}
                    disabled={importing.has(key)}
                    style={{ opacity: importing.has(key) ? 0.6 : 1 }}
                  >
                    {importing.has(key) ? strings.installing : strings.install}
                  </Button>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
