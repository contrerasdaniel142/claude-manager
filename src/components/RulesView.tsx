import { useEffect, useState } from "react";
import { api } from "../api";
import type { RuleSummary, Scope } from "../types";
import AiAssistBox from "./AiAssistBox";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import MarkdownEditor from "./MarkdownEditor";
import TagInput from "./TagInput";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";
import { FILE_GLOB_SUGGESTIONS } from "../toolSuggestions";

interface Props {
  scope: Scope;
  onListChanged: (list: RuleSummary[]) => void;
  refreshSignal: number;
  /** Set by the Advisor module when it sends the user here with fields it
   * already generated from a described need — applied as if this view's
   * own AiAssistBox had just returned that same result. */
  prefill?: Record<string, unknown>;
  onPrefillConsumed?: () => void;
  /** Set by Global Search to jump straight to an existing rule. */
  openId?: string;
  onOpenIdConsumed?: () => void;
}

type RuleForm = { id: string; paths: string[]; body: string };
const emptyForm = (): RuleForm => ({ id: "", paths: [], body: "" });

function mergeAiResult(base: RuleForm, data: Record<string, unknown>): RuleForm {
  return {
    id: typeof data.id === "string" ? data.id : base.id,
    paths: Array.isArray(data.paths) ? (data.paths as string[]) : base.paths,
    body: typeof data.body === "string" ? data.body : base.body,
  };
}

export default function RulesView({
  scope,
  onListChanged,
  refreshSignal,
  prefill,
  onPrefillConsumed,
  openId,
  onOpenIdConsumed,
}: Props) {
  const [list, setList] = useState<RuleSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [originalId, setOriginalId] = useState<string | null>(null);
  const [form, setForm] = useState<RuleForm>(emptyForm());
  const toast = useToast();
  const { t } = useLanguage();

  const refresh = async () => {
    const items = await api.listRules(scope);
    setList(items);
    onListChanged(items);
  };

  useEffect(() => {
    refresh();
    setSelected(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  // Must run after the `[scope]` effect above — both fire on mount, in
  // declaration order, and that one unconditionally resets `selected`,
  // which would wipe this one's "__new__" if it ran first.
  useEffect(() => {
    if (!prefill) return;
    setSelected("__new__");
    setOriginalId(null);
    setForm(mergeAiResult(emptyForm(), prefill));
    onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [prefill]);

  // Un cambio hecho por fuera de la app (editar a mano, git pull, etc.) —
  // solo refresca la lista, sin tocar lo que se esté editando ahora mismo.
  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  const openNew = () => {
    setSelected("__new__");
    setOriginalId(null);
    setForm(emptyForm());
  };

  const openExisting = async (id: string) => {
    const rule = await api.readRule(scope, id);
    setSelected(id);
    setOriginalId(id);
    setForm({ id: rule.id, paths: rule.paths, body: rule.body });
  };

  useEffect(() => {
    if (!openId) return;
    openExisting(openId);
    onOpenIdConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openId]);

  const save = async () => {
    try {
      const summary = await api.saveRule(scope, originalId, form.id.trim(), form.paths, form.body);
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
    if (!window.confirm(t.rules.deleteConfirm(id))) return;
    await api.deleteRule(scope, id);
    setSelected(null);
    await refresh();
  };

  const moveToOtherScope = async () => {
    if (!originalId) return;
    const otherScope: Scope = scope === "project" ? "global" : "project";
    try {
      await api.saveRule(otherScope, null, form.id.trim(), form.paths, form.body);
      await api.deleteRule(scope, originalId);
      toast.success(t.common.movedToast(form.id, otherScope === "project" ? t.app.scopeProject : t.app.scopeGlobal));
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
            {t.rules.newRule}
          </Button>
        </div>
        {list.map((r) => (
          <ListRow
            key={r.id}
            active={selected === r.id}
            onClick={() => openExisting(r.id)}
            title={r.title}
            subtitle={r.paths.length > 0 ? r.paths.join(", ") : t.rules.alwaysLoaded}
          />
        ))}
        {list.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {t.rules.emptyList}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {!selected ? (
          <div className="max-w-2xl" style={{ color: "var(--text-muted)" }}>
            <p className="mb-2">{t.rules.emptyMainP1}</p>
            <p className="text-xs">{t.rules.emptyMainP2(".claude/rules/*.md", "paths", "fileMatchPattern")}</p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-4">

            <AiAssistBox
              kind="rule"
              currentFields={form}
              onResult={(data) => setForm((prev) => mergeAiResult(prev, data))}
            />

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.rules.fileNameLabel}
              </label>
              <TextInput
                value={form.id}
                onChange={(e) => setForm({ ...form, id: e.target.value })}
                placeholder="clean-architecture-java"
                className="w-full mt-1"
              />
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.rules.fileNameHint(`.claude/rules/${form.id || "…"}.md`)}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.rules.pathsLabel}
              </label>
              <div className="mt-1">
                <TagInput
                  tags={form.paths}
                  onChange={(paths) => setForm({ ...form, paths })}
                  suggestions={FILE_GLOB_SUGGESTIONS}
                  placeholder={`${t.common.eg}: src/**/*.java`}
                />
              </div>
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.rules.pathsHint}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.rules.contentLabel}
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
