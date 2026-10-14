import { useEffect, useState } from "react";
import { api } from "../api";
import type { OutputStyleSummary, Scope } from "../types";
import AiAssistBox from "./AiAssistBox";
import Button from "./Button";
import Checkbox from "./Checkbox";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import MarkdownEditor from "./MarkdownEditor";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";

interface Props {
  scope: Scope;
  onListChanged: (list: OutputStyleSummary[]) => void;
  refreshSignal: number;
  /** Set by the Advisor module when it sends the user here with fields it
   * already generated from a described need — applied as if this view's
   * own AiAssistBox had just returned that same result. */
  prefill?: Record<string, unknown>;
  onPrefillConsumed?: () => void;
  /** Set by Global Search to jump straight to an existing output style. */
  openId?: string;
  onOpenIdConsumed?: () => void;
}

const emptyForm = () => ({
  id: "",
  name: "",
  description: "",
  keepCodingInstructions: false,
  body: "",
});

type OutputStyleForm = ReturnType<typeof emptyForm>;

function mergeAiResult(base: OutputStyleForm, data: Record<string, unknown>): OutputStyleForm {
  return {
    id: typeof data.id === "string" ? data.id : base.id,
    name: typeof data.name === "string" ? data.name : base.name,
    description: typeof data.description === "string" ? data.description : base.description,
    keepCodingInstructions:
      typeof data.keepCodingInstructions === "boolean" ? data.keepCodingInstructions : base.keepCodingInstructions,
    body: typeof data.body === "string" ? data.body : base.body,
  };
}

export default function OutputStylesView({
  scope,
  onListChanged,
  refreshSignal,
  prefill,
  onPrefillConsumed,
  openId,
  onOpenIdConsumed,
}: Props) {
  const [list, setList] = useState<OutputStyleSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [originalId, setOriginalId] = useState<string | null>(null);
  const [form, setForm] = useState(emptyForm());
  const toast = useToast();
  const { t } = useLanguage();

  const refresh = async () => {
    const items = await api.listOutputStyles(scope);
    setList(items);
    onListChanged(items);
  };

  useEffect(() => {
    refresh();
    setSelected(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  // Must run after the `[scope]` effect above — see RulesView for why.
  useEffect(() => {
    if (!prefill) return;
    setSelected("__new__");
    setOriginalId(null);
    setForm(mergeAiResult(emptyForm(), prefill));
    onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [prefill]);

  const openNew = () => {
    setSelected("__new__");
    setOriginalId(null);
    setForm(emptyForm());
  };

  const openExisting = async (id: string) => {
    const style = await api.readOutputStyle(scope, id);
    setSelected(id);
    setOriginalId(id);
    setForm({
      id: style.id,
      name: style.name,
      description: style.description,
      keepCodingInstructions: style.keep_coding_instructions,
      body: style.body,
    });
  };

  useEffect(() => {
    if (!openId) return;
    openExisting(openId);
    onOpenIdConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openId]);

  const save = async () => {
    try {
      const summary = await api.saveOutputStyle(
        scope,
        originalId,
        form.id.trim(),
        form.name.trim() || null,
        form.description.trim() || null,
        form.keepCodingInstructions,
        form.body,
      );
      toast.success(t.common.saved);
      await refresh();
      setOriginalId(summary.id);
      setSelected(summary.id);
      setForm({ ...form, id: summary.id });
    } catch (e) {
      toast.error(String(e));
    }
  };

  const remove = async (id: string) => {
    if (!window.confirm(t.outputStyles.deleteConfirm(id))) return;
    await api.deleteOutputStyle(scope, id);
    setSelected(null);
    await refresh();
  };

  const moveToOtherScope = async () => {
    if (!originalId) return;
    const otherScope: Scope = scope === "project" ? "global" : "project";
    try {
      await api.saveOutputStyle(
        otherScope,
        null,
        form.id.trim(),
        form.name.trim() || null,
        form.description.trim() || null,
        form.keepCodingInstructions,
        form.body,
      );
      await api.deleteOutputStyle(scope, originalId);
      toast.success(
        t.common.movedToast(form.name || originalId, otherScope === "project" ? t.app.scopeProject : t.app.scopeGlobal),
      );
      setSelected(null);
      await refresh();
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="flex h-full">
      <div className="w-72 shrink-0 border-r overflow-y-auto" style={{ borderColor: "var(--border)" }}>
        <div className="p-3">
          <Button full onClick={openNew}>
            {t.outputStyles.newStyle}
          </Button>
        </div>
        {list.map((s) => (
          <ListRow
            key={s.id}
            active={selected === s.id}
            onClick={() => openExisting(s.id)}
            title={s.name}
            subtitle={s.description || t.outputStyles.noDescription}
          />
        ))}
        {list.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {t.outputStyles.emptyList}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {!selected ? (
          <div className="max-w-2xl space-y-3" style={{ color: "var(--text-muted)" }}>
            <p>{t.outputStyles.emptyMainP1}</p>
            <p className="text-xs">{t.outputStyles.emptyMainP2}</p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-4">
            <AiAssistBox
              kind="outputStyle"
              currentFields={form}
              onResult={(data) => setForm((prev) => mergeAiResult(prev, data))}
            />

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.outputStyles.fileNameLabel}
              </label>
              <TextInput
                value={form.id}
                onChange={(e) => setForm({ ...form, id: e.target.value })}
                placeholder="diagrams-first"
                className="w-full mt-1"
              />
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.outputStyles.fileNameHint(`.claude/output-styles/${form.id || "…"}.md`)}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.outputStyles.nameLabel}
              </label>
              <TextInput
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder={t.outputStyles.namePlaceholder}
                className="w-full mt-1"
              />
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.outputStyles.descriptionLabel}
              </label>
              <TextInput
                value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })}
                placeholder={t.outputStyles.descriptionPlaceholder}
                className="w-full mt-1"
              />
            </div>

            <div>
              <Checkbox
                label={t.outputStyles.keepCodingInstructionsLabel}
                checked={form.keepCodingInstructions}
                onChange={(e) => setForm({ ...form, keepCodingInstructions: e.target.checked })}
              />
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.outputStyles.keepCodingInstructionsHint}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.outputStyles.contentLabel}
              </label>
              <div className="mt-1">
                <MarkdownEditor value={form.body} onChange={(v) => setForm({ ...form, body: v })} minRows={16} />
              </div>
            </div>

            <div className="flex gap-2 pt-2">
              <Button onClick={save}>{t.common.save}</Button>
              {originalId && (
                <>
                  <Button variant="ghost" onClick={moveToOtherScope}>
                    {scope === "project" ? t.common.moveToGlobal : t.common.moveToProject}
                  </Button>
                  <Button variant="danger" onClick={() => remove(originalId)}>
                    {t.common.delete}
                  </Button>
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
