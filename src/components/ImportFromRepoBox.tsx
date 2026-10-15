import { useState } from "react";
import ReactMarkdown from "react-markdown";
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
  /** Optional preview functions and labels for expandable row content. */
  getPreviewBody?: (item: T) => string | null;
  getPreviewWhenToUse?: (item: T) => string | null;
  previewLabels?: { whenToUseLabel: string; emptyBody: string };
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
  getPreviewBody,
  getPreviewWhenToUse,
  previewLabels,
}: Props<T>) {
  const [repoUrl, setRepoUrl] = useState("");
  const [found, setFound] = useState<T[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [importing, setImporting] = useState<Set<string>>(new Set());
  const [expanded, setExpanded] = useState<string | null>(null);
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

  const toggleExpand = (key: string) => {
    setExpanded((curr) => (curr === key ? null : key));
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
            const hasPreview = !!getPreviewBody;
            const isExpanded = hasPreview && expanded === key;
            const whenToUse = getPreviewWhenToUse ? getPreviewWhenToUse(item) : null;
            const rawBody = getPreviewBody ? getPreviewBody(item) : null;
            const body = rawBody && rawBody.trim() ? rawBody : previewLabels?.emptyBody || "";

            return (
              <div
                key={key}
                className={hasPreview ? "rounded-lg border" : "p-2.5 rounded-lg border flex items-start gap-3"}
                style={{ borderColor: "var(--border)" }}
              >
                <div
                  {...(hasPreview
                    ? {
                        role: "button",
                        tabIndex: 0,
                        onClick: () => toggleExpand(key),
                        onKeyDown: (e: React.KeyboardEvent) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            toggleExpand(key);
                          }
                        },
                        className: "w-full p-2.5 flex items-start gap-3 text-left cursor-pointer",
                      }
                    : {
                        className: "contents",
                      })}
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
                      onClick={(e) => {
                        e.stopPropagation();
                        install(item);
                      }}
                      disabled={importing.has(key)}
                      style={{ opacity: importing.has(key) ? 0.6 : 1 }}
                    >
                      {importing.has(key) ? strings.installing : strings.install}
                    </Button>
                  )}
                </div>

                {hasPreview && isExpanded && (
                  <div className="px-3 pb-3 border-t pt-2 space-y-2" style={{ borderColor: "var(--border)" }}>
                    {whenToUse && previewLabels?.whenToUseLabel && (
                      <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                        {previewLabels.whenToUseLabel}: {whenToUse}
                      </p>
                    )}
                    <div
                      className="prose prose-sm max-w-none max-h-64 overflow-auto rounded-md border p-2"
                      style={{ borderColor: "var(--border)", color: "var(--text)" }}
                    >
                      <ReactMarkdown>{body}</ReactMarkdown>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
