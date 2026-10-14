import { useEffect, useState } from "react";
import { api } from "../api";
import type { AdvisorCandidate, AdvisorKind, Category } from "../types";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import Select from "./Select";
import TextArea from "./TextArea";
import { useToast } from "./ToastProvider";

interface Props {
  onNavigate: (category: Category, prefill: Record<string, unknown> | null) => void;
}

const KIND_TO_CATEGORY: Record<AdvisorKind, Category> = {
  rule: "rules",
  hook: "hooks",
  skill: "skills",
  outputStyle: "outputStyles",
  permissions: "permissions",
  mcpServer: "mcpServers",
  plugin: "plugins",
};

export default function AdvisorView({ onNavigate }: Props) {
  const [description, setDescription] = useState("");
  const [model, setModel] = useState("haiku");
  const [models, setModels] = useState<string[]>([]);
  const [analyzing, setAnalyzing] = useState(false);
  const [candidates, setCandidates] = useState<AdvisorCandidate[] | null>(null);
  const [applying, setApplying] = useState<AdvisorKind | null>(null);
  const toast = useToast();
  const { t } = useLanguage();

  useEffect(() => {
    api.listAssistantModels().then(setModels);
  }, []);

  const analyze = async () => {
    if (!description.trim()) return;
    setAnalyzing(true);
    setCandidates(null);
    try {
      const data = await api.generateWithAI("advisor", description.trim(), {}, model);
      const result = Array.isArray(data.candidates) ? (data.candidates as AdvisorCandidate[]) : [];
      setCandidates(result);
      if (result.length === 1) {
        await useCandidate(result[0]);
      }
    } catch (e) {
      toast.error(String(e));
    } finally {
      setAnalyzing(false);
    }
  };

  const useCandidate = async (candidate: AdvisorCandidate) => {
    const category = KIND_TO_CATEGORY[candidate.kind];
    if (candidate.kind === "plugin") {
      toast.success(t.advisor.pluginNote);
      onNavigate(category, null);
      return;
    }
    setApplying(candidate.kind);
    try {
      const fields = await api.generateWithAI(candidate.kind, description.trim(), {}, model);
      onNavigate(category, fields);
      toast.success(t.advisor.navigatedToast(t.app.categories[category]));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setApplying(null);
    }
  };

  return (
    <div className="h-full overflow-y-auto p-5">
      <div className="max-w-2xl space-y-4">
        <p style={{ color: "var(--text-muted)" }}>{t.advisor.intro}</p>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.advisor.descriptionLabel}
          </label>
          <TextArea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t.advisor.descriptionPlaceholder}
            rows={4}
            className="w-full mt-1"
          />
        </div>

        <div className="flex gap-2 items-center">
          <Select value={model} onChange={(e) => setModel(e.target.value)} compact>
            {models.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </Select>
          <Button onClick={analyze} disabled={analyzing || !description.trim()}>
            {analyzing ? t.advisor.analyzing : t.advisor.analyze}
          </Button>
        </div>

        {candidates && candidates.length > 1 && (
          <div className="space-y-2">
            <div className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
              {t.advisor.resultsIntro}
            </div>
            {candidates.map((c) => (
              <div
                key={c.kind}
                className="p-3 rounded-lg border flex items-start gap-3"
                style={{ borderColor: "var(--border)" }}
              >
                <div className="min-w-0 flex-1">
                  <div className="text-sm font-medium">{t.app.categories[KIND_TO_CATEGORY[c.kind]]}</div>
                  <div className="text-xs mt-0.5" style={{ color: "var(--text-muted)" }}>
                    {c.reason}
                  </div>
                </div>
                <Button
                  size="sm"
                  className="shrink-0"
                  disabled={applying !== null}
                  onClick={() => useCandidate(c)}
                >
                  {applying === c.kind ? t.advisor.preparing : t.advisor.useThis}
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
