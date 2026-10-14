import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import { api } from "../api";
import type { MarketplaceSkillPreview, MarketplaceSkillSummary, Scope } from "../types";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";

interface Props {
  scope: Scope;
  /** Names already installed locally — used to grey out a match instead of
   * letting it collide on install. */
  localNames: string[];
  onInstalled: () => void;
}

/** Search box for skills.sh (the community skill index also used by its own
 * `skills` CLI) — debounced search-as-you-type against its lightweight
 * `/api/search` endpoint, which returns at most ~100 light results per query
 * instead of a catalog dump, so there's nothing to paginate client-side. */
export default function SkillMarketplaceBox({ scope, localNames, onInstalled }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<MarketplaceSkillSummary[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<Set<string>>(new Set());
  const [expanded, setExpanded] = useState<string | null>(null);
  const [previews, setPreviews] = useState<Record<string, MarketplaceSkillPreview>>({});
  const [previewErrors, setPreviewErrors] = useState<Record<string, string>>({});
  const [previewLoading, setPreviewLoading] = useState<string | null>(null);
  const toast = useToast();
  const { t } = useLanguage();
  const requestId = useRef(0);

  useEffect(() => {
    const q = query.trim();
    if (q.length < 2) {
      setResults(null);
      return;
    }
    const id = ++requestId.current;
    setLoading(true);
    const timer = setTimeout(() => {
      api
        .searchMarketplaceSkills(q)
        .then((r) => {
          if (id === requestId.current) {
            setResults(r);
            setLoading(false);
          }
        })
        .catch((e) => {
          if (id === requestId.current) {
            toast.error(String(e));
            setLoading(false);
          }
        });
    }, 250);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query]);

  const toggleExpand = async (item: MarketplaceSkillSummary) => {
    const key = `${item.source}::${item.skill_id}`;
    if (expanded === key) {
      setExpanded(null);
      return;
    }
    setExpanded(key);
    if (previews[key] || previewErrors[key]) return; // already tried once, don't re-hit GitHub
    setPreviewLoading(key);
    try {
      const preview = await api.previewMarketplaceSkill(item.source, item.skill_id);
      setPreviews((prev) => ({ ...prev, [key]: preview }));
    } catch (e) {
      // Shown inline instead of a toast-and-collapse: the user needs to see
      // *why* this one can't be previewed to judge whether Install (which
      // doesn't go through this same path) is even worth trying.
      setPreviewErrors((prev) => ({ ...prev, [key]: String(e) }));
    } finally {
      setPreviewLoading((current) => (current === key ? null : current));
    }
  };

  const install = async (item: MarketplaceSkillSummary) => {
    const key = `${item.source}::${item.skill_id}`;
    setInstalling((prev) => new Set(prev).add(key));
    try {
      await api.installMarketplaceSkill(item.source, item.skill_id, scope);
      onInstalled();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setInstalling((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  return (
    <div className="max-w-2xl space-y-3">
      <p style={{ color: "var(--text-muted)" }}>{t.skillMarketplace.intro}</p>
      <TextInput
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={t.skillMarketplace.searchPlaceholder}
        className="w-full"
      />

      {loading && <p style={{ color: "var(--text-muted)" }}>{t.skillMarketplace.searching}</p>}
      {!loading && results && results.length === 0 && (
        <p style={{ color: "var(--text-muted)" }}>{t.skillMarketplace.noneFound}</p>
      )}

      {!loading && results && results.length > 0 && (
        <div className="space-y-1.5">
          {results.map((item) => {
            const key = `${item.source}::${item.skill_id}`;
            const alreadyInstalled = localNames.includes(item.skill_id) || localNames.includes(item.name);
            const isExpanded = expanded === key;
            const preview = previews[key];
            const previewError = previewErrors[key];
            return (
              <div key={key} className="rounded-lg border" style={{ borderColor: "var(--border)" }}>
                <div
                  role="button"
                  tabIndex={0}
                  onClick={() => toggleExpand(item)}
                  onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && toggleExpand(item)}
                  className="w-full p-2.5 flex items-start gap-3 text-left cursor-pointer"
                >
                  <div className="min-w-0 flex-1">
                    <div className="text-sm font-medium truncate">{item.name}</div>
                    <div className="text-xs mt-0.5" style={{ color: "var(--text-muted)" }}>
                      {item.source} — {t.skillMarketplace.installsCount(item.installs)}
                    </div>
                  </div>
                  {alreadyInstalled ? (
                    <span className="shrink-0 text-xs pt-0.5" style={{ color: "var(--text-muted)" }}>
                      {t.skillMarketplace.alreadyInstalled}
                    </span>
                  ) : (
                    <Button
                      size="sm"
                      className="shrink-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        install(item);
                      }}
                      disabled={installing.has(key)}
                      style={{ opacity: installing.has(key) ? 0.6 : 1 }}
                    >
                      {installing.has(key) ? t.skillMarketplace.installing : t.skillMarketplace.install}
                    </Button>
                  )}
                </div>
                {isExpanded && (
                  <div className="px-3 pb-3 border-t pt-2" style={{ borderColor: "var(--border)" }}>
                    {previewLoading === key && (
                      <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                        {t.skillMarketplace.loadingPreview}
                      </p>
                    )}
                    {previewError && (
                      <div className="space-y-1">
                        <p className="text-xs" style={{ color: "var(--danger)" }}>
                          {previewError}
                        </p>
                        <p className="text-[11px]" style={{ color: "var(--text-muted)" }}>
                          {t.skillMarketplace.previewFailedInstallHint}
                        </p>
                      </div>
                    )}
                    {preview && (
                      <div className="space-y-2">
                        {preview.description && <p className="text-sm">{preview.description}</p>}
                        {preview.when_to_use && (
                          <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                            {t.skillMarketplace.whenToUseLabel}: {preview.when_to_use}
                          </p>
                        )}
                        <div
                          className="prose prose-sm max-w-none max-h-64 overflow-auto rounded-md border p-2"
                          style={{ borderColor: "var(--border)", color: "var(--text)" }}
                        >
                          <ReactMarkdown>{preview.body || t.skillMarketplace.emptyBody}</ReactMarkdown>
                        </div>
                      </div>
                    )}
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
