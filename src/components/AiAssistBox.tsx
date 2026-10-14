import { useEffect, useState } from "react";
import { api } from "../api";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import Select from "./Select";
import TextArea from "./TextArea";
import { useToast } from "./ToastProvider";

interface Props {
  kind: "rule" | "hook" | "agent" | "skill" | "claudemd" | "outputStyle" | "permissions" | "mcpServer" | "advisor";
  currentFields: unknown;
  onResult: (data: Record<string, unknown>) => void;
  /** Si no se pasa, usa t.aiAssist.generateWithClaude. */
  label?: string;
  /** Si está, no hay botón disparador propio — siempre muestra el formulario
   * expandido. Para cuando otro botón (en otro panel) es el que decide
   * mostrar este componente, como "Instalar desde repo" en Skills. */
  forceOpen?: boolean;
  /** Qué hacer al tocar "Cancelar" cuando forceOpen está activo (el panel
   * que lo muestra decide, ej. volver al estado vacío). Sin esto, cancelar
   * solo colapsa el formulario de vuelta al botón disparador propio. */
  onCancel?: () => void;
}

export default function AiAssistBox({
  kind,
  currentFields,
  onResult,
  label,
  forceOpen = false,
  onCancel,
}: Props) {
  const [available, setAvailable] = useState<boolean | null>(null);
  const [models, setModels] = useState<string[]>([]);
  const [model, setModel] = useState("haiku");
  const [open, setOpen] = useState(forceOpen);
  const [prompt, setPrompt] = useState("");
  const [loading, setLoading] = useState(false);
  const toast = useToast();
  const { t } = useLanguage();
  const effectiveLabel = label ?? t.aiAssist.generateWithClaude;

  useEffect(() => {
    api.assistantAvailable().then(setAvailable);
    api.listAssistantModels().then(setModels);
  }, []);

  if (available === false) return null;

  const generate = async () => {
    if (!prompt.trim()) return;
    setLoading(true);
    try {
      const data = await api.generateWithAI(kind, prompt.trim(), currentFields, model);
      onResult(data);
      setOpen(false);
      setPrompt("");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
    }
  };

  if (!open) {
    return (
      <Button full onClick={() => setOpen(true)} disabled={available === null}>
        {effectiveLabel}
      </Button>
    );
  }

  return (
    <div className="rounded-lg border p-3 space-y-2" style={{ borderColor: "var(--accent)", background: "var(--accent-soft)" }}>
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs font-medium" style={{ color: "var(--accent)" }}>
          {t.aiAssist.describe}
        </div>
        <Select value={model} onChange={(e) => setModel(e.target.value)} title={t.aiAssist.modelTitle} compact>
          {models.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </Select>
      </div>
      <TextArea
        value={prompt}
        onChange={(e) => setPrompt(e.target.value)}
        rows={2}
        autoFocus
        placeholder={t.aiAssist.promptPlaceholder}
        className="w-full"
        onKeyDown={(e) => {
          if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) generate();
        }}
      />
      <div className="flex gap-2">
        <Button size="sm" onClick={generate} disabled={loading || !prompt.trim()} style={{ opacity: loading ? 0.6 : 1 }}>
          {loading ? t.aiAssist.generating : t.aiAssist.generate}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => {
            if (onCancel) onCancel();
            else setOpen(false);
          }}
        >
          {t.aiAssist.cancel}
        </Button>
      </div>
    </div>
  );
}
