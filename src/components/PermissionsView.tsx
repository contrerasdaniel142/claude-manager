import { useEffect, useState } from "react";
import { api } from "../api";
import type { PermissionsSettings, Scope } from "../types";
import AiAssistBox from "./AiAssistBox";
import BoolSelect from "./BoolSelect";
import Button from "./Button";
import Checkbox from "./Checkbox";
import { useLanguage } from "./LanguageProvider";
import Select from "./Select";
import TagInput from "./TagInput";
import { useToast } from "./ToastProvider";
import { BUILTIN_TOOLS } from "../toolSuggestions";

type TriBool = "" | "true" | "false";
const triToBool = (v: TriBool): boolean | null => (v === "" ? null : v === "true");
const boolToTri = (v: boolean | null | undefined): TriBool => (v === true ? "true" : v === false ? "false" : "");

/** attributionCommit/Pr are `null` (leave the default, which adds the
 * trailer) or a string — usually `""` to hide it entirely. A plain text
 * input can't tell "untouched" apart from "explicitly emptied", so this is
 * a 2-option choice instead; a custom trailer already saved by hand stays
 * intact until the user picks one of these two options themselves. */
const UNSET = "__unset__";

interface Props {
  scope: Scope;
  onCountChanged: (count: number) => void;
  refreshSignal: number;
  /** Set by the Advisor module when it sends the user here with fields it
   * already generated from a described need — applied as if this view's
   * own AiAssistBox had just returned that same result, merged onto
   * whatever permissions are already saved (there's no "new item" here). */
  prefill?: Record<string, unknown>;
  onPrefillConsumed?: () => void;
}

const emptyForm = (): PermissionsSettings => ({
  allow: [],
  ask: [],
  deny: [],
  additionalDirectories: [],
  defaultMode: null,
  disableBypassPermissionsMode: false,
  disableAutoMode: false,
  blockReadsOutsideWorkingDirectories: false,
  attributionCommit: null,
  attributionPr: null,
  attributionSessionUrl: null,
  includeGitInstructions: null,
});

function mergeAiResult(base: PermissionsSettings, data: Record<string, unknown>): PermissionsSettings {
  return {
    allow: Array.isArray(data.allow) ? (data.allow as string[]) : base.allow,
    ask: Array.isArray(data.ask) ? (data.ask as string[]) : base.ask,
    deny: Array.isArray(data.deny) ? (data.deny as string[]) : base.deny,
    additionalDirectories: Array.isArray(data.additionalDirectories)
      ? (data.additionalDirectories as string[])
      : base.additionalDirectories,
    defaultMode:
      typeof data.defaultMode === "string" || data.defaultMode === null
        ? (data.defaultMode as string | null)
        : base.defaultMode,
    disableBypassPermissionsMode:
      typeof data.disableBypassPermissionsMode === "boolean"
        ? data.disableBypassPermissionsMode
        : base.disableBypassPermissionsMode,
    disableAutoMode: typeof data.disableAutoMode === "boolean" ? data.disableAutoMode : base.disableAutoMode,
    blockReadsOutsideWorkingDirectories:
      typeof data.blockReadsOutsideWorkingDirectories === "boolean"
        ? data.blockReadsOutsideWorkingDirectories
        : base.blockReadsOutsideWorkingDirectories,
    attributionCommit:
      typeof data.attributionCommit === "string" || data.attributionCommit === null
        ? (data.attributionCommit as string | null)
        : base.attributionCommit,
    attributionPr:
      typeof data.attributionPr === "string" || data.attributionPr === null
        ? (data.attributionPr as string | null)
        : base.attributionPr,
    attributionSessionUrl:
      typeof data.attributionSessionUrl === "boolean" || data.attributionSessionUrl === null
        ? (data.attributionSessionUrl as boolean | null)
        : base.attributionSessionUrl,
    includeGitInstructions:
      typeof data.includeGitInstructions === "boolean" || data.includeGitInstructions === null
        ? (data.includeGitInstructions as boolean | null)
        : base.includeGitInstructions,
  };
}

export default function PermissionsView({ scope, onCountChanged, refreshSignal, prefill, onPrefillConsumed }: Props) {
  const [form, setForm] = useState<PermissionsSettings>(emptyForm());
  // `refresh` below is async — on a fresh mount its `setForm(data)` always
  // lands in a later tick than the prefill effect's synchronous one, no
  // matter which effect is declared first. This flag makes the prefill
  // effect wait for the real load to land before merging onto it, instead
  // of merging onto a still-empty form that the load then overwrites.
  const [loaded, setLoaded] = useState(false);
  const toast = useToast();
  const { t } = useLanguage();

  const refresh = async () => {
    const data = await api.readPermissions(scope);
    setForm(data);
    setLoaded(true);
    onCountChanged(data.allow.length + data.ask.length + data.deny.length);
  };

  useEffect(() => {
    setLoaded(false);
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  useEffect(() => {
    if (!prefill || !loaded) return;
    setForm((prev) => mergeAiResult(prev, prefill));
    onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [prefill, loaded]);

  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  const save = async () => {
    try {
      const saved = await api.savePermissions(scope, form);
      setForm(saved);
      onCountChanged(saved.allow.length + saved.ask.length + saved.deny.length);
      toast.success(t.permissions.savedToast);
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="h-full overflow-y-auto p-5">
      <div className="max-w-2xl space-y-4">
        <AiAssistBox
          kind="permissions"
          currentFields={form}
          onResult={(data) => setForm((prev) => mergeAiResult(prev, data))}
        />

        <p className="text-xs" style={{ color: "var(--text-muted)" }}>
          {t.permissions.intro}
        </p>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.allowLabel}
          </label>
          <div className="mt-1">
            <TagInput
              tags={form.allow}
              onChange={(allow) => setForm({ ...form, allow })}
              suggestions={BUILTIN_TOOLS}
              placeholder={t.permissions.rulesPlaceholder}
            />
          </div>
        </div>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.askLabel}
          </label>
          <div className="mt-1">
            <TagInput
              tags={form.ask}
              onChange={(ask) => setForm({ ...form, ask })}
              suggestions={BUILTIN_TOOLS}
              placeholder={t.permissions.rulesPlaceholder}
            />
          </div>
        </div>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.denyLabel}
          </label>
          <div className="mt-1">
            <TagInput
              tags={form.deny}
              onChange={(deny) => setForm({ ...form, deny })}
              suggestions={BUILTIN_TOOLS}
              placeholder={t.permissions.rulesPlaceholder}
            />
          </div>
        </div>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.additionalDirectoriesLabel}
          </label>
          <div className="mt-1">
            <TagInput
              tags={form.additionalDirectories}
              onChange={(additionalDirectories) => setForm({ ...form, additionalDirectories })}
              suggestions={[]}
              placeholder={t.permissions.additionalDirectoriesPlaceholder}
            />
          </div>
        </div>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.defaultModeLabel}
          </label>
          <Select
            value={form.defaultMode ?? ""}
            onChange={(e) => setForm({ ...form, defaultMode: e.target.value || null })}
            className="w-full mt-1"
          >
            <option value="">{t.permissions.defaultModeUnset}</option>
            {["default", "acceptEdits", "plan", "auto", "dontAsk", "bypassPermissions"].map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </Select>
        </div>

        <Checkbox
          label={t.permissions.disableBypassPermissionsModeLabel}
          checked={form.disableBypassPermissionsMode}
          onChange={(e) => setForm({ ...form, disableBypassPermissionsMode: e.target.checked })}
        />
        <Checkbox
          label={t.permissions.disableAutoModeLabel}
          checked={form.disableAutoMode}
          onChange={(e) => setForm({ ...form, disableAutoMode: e.target.checked })}
        />
        <Checkbox
          label={t.permissions.blockReadsOutsideWorkingDirectoriesLabel}
          checked={form.blockReadsOutsideWorkingDirectories}
          onChange={(e) => setForm({ ...form, blockReadsOutsideWorkingDirectories: e.target.checked })}
        />

        <div className="pt-2 border-t" style={{ borderColor: "var(--border)" }}>
          <h2 className="text-sm font-semibold mt-3">{t.permissions.attributionTitle}</h2>
          <p className="text-xs mt-1" style={{ color: "var(--text-muted)" }}>
            {t.permissions.attributionIntro}
          </p>
        </div>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.attributionCommitLabel}
          </label>
          <Select
            value={form.attributionCommit === null ? UNSET : form.attributionCommit}
            onChange={(e) =>
              setForm({ ...form, attributionCommit: e.target.value === UNSET ? null : e.target.value })
            }
            className="w-full mt-1"
          >
            <option value={UNSET}>{t.permissions.attributionDefault}</option>
            <option value="">{t.permissions.attributionHide}</option>
          </Select>
        </div>

        <div>
          <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
            {t.permissions.attributionPrLabel}
          </label>
          <Select
            value={form.attributionPr === null ? UNSET : form.attributionPr}
            onChange={(e) => setForm({ ...form, attributionPr: e.target.value === UNSET ? null : e.target.value })}
            className="w-full mt-1"
          >
            <option value={UNSET}>{t.permissions.attributionDefault}</option>
            <option value="">{t.permissions.attributionHide}</option>
          </Select>
        </div>

        <BoolSelect
          label={t.permissions.attributionSessionUrlLabel}
          value={boolToTri(form.attributionSessionUrl)}
          onChange={(v) => setForm({ ...form, attributionSessionUrl: triToBool(v) })}
          hint={t.permissions.attributionSessionUrlHint}
        />

        <BoolSelect
          label={t.permissions.includeGitInstructionsLabel}
          value={boolToTri(form.includeGitInstructions)}
          onChange={(v) => setForm({ ...form, includeGitInstructions: triToBool(v) })}
          hint={t.permissions.includeGitInstructionsHint}
        />

        <div className="flex gap-2 pt-2">
          <Button onClick={save}>{t.common.save}</Button>
        </div>
      </div>
    </div>
  );
}
