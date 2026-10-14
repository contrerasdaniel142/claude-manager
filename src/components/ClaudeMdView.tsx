import { useEffect, useState } from "react";
import { api } from "../api";
import type { Scope, SteeringSummary } from "../types";
import AiAssistBox from "./AiAssistBox";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import MarkdownEditor from "./MarkdownEditor";
import { useToast } from "./ToastProvider";

interface Props {
  scope: Scope;
  onListChanged: (list: SteeringSummary[]) => void;
  refreshSignal: number;
}

export default function ClaudeMdView({ scope, onListChanged, refreshSignal }: Props) {
  const [list, setList] = useState<SteeringSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const toast = useToast();
  const { t } = useLanguage();

  /** "Principal"/"Main" for the root CLAUDE.md, the containing folder's name
   * for any other one, or "Global" for the single ~/.claude/CLAUDE.md — this
   * is what tells two entries named CLAUDE.md apart in the list. */
  const folderLabel = (relPath: string): string => {
    if (scope === "global") return t.app.scopeGlobal;
    if (relPath === "CLAUDE.md") return t.claudeMd.principal;
    const parts = relPath.split("/");
    return parts[parts.length - 2] ?? relPath;
  };

  const refresh = async () => {
    const items = await api.listSteering(scope);
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

  const open = async (relPath: string) => {
    const file = await api.readSteering(scope, relPath);
    setSelected(relPath);
    setContent(file.content);
  };

  const save = async () => {
    if (!selected) return;
    try {
      await api.saveSteering(scope, selected, content);
      toast.success(t.common.saved);
      await refresh();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const createRootWithAi = async (data: Record<string, unknown>) => {
    if (typeof data.content !== "string") return;
    try {
      await api.saveSteering(scope, "CLAUDE.md", data.content);
      await refresh();
      await open("CLAUDE.md");
    } catch (e) {
      toast.error(String(e));
    }
  };

  const remove = async (relPath: string) => {
    if (!window.confirm(t.claudeMd.deleteConfirm(relPath))) return;
    await api.deleteSteering(scope, relPath);
    setSelected(null);
    await refresh();
  };

  const hasRoot = list.some((s) => s.rel_path === "CLAUDE.md");

  return (
    <div className="flex h-full">
      <div className="w-72 shrink-0 border-r overflow-y-auto" style={{ borderColor: "var(--border)" }}>
        {!hasRoot && (
          <div className="p-3">
            <Button full onClick={() => setSelected("__generate__")}>
              {t.claudeMd.generateWithClaude}
            </Button>
          </div>
        )}
        {list.map((s) => (
          <ListRow
            key={s.rel_path}
            active={selected === s.rel_path}
            onClick={() => open(s.rel_path)}
            title={folderLabel(s.rel_path)}
            subtitle={`${s.rel_path} · ${(s.size / 1024).toFixed(1)} KB`}
          />
        ))}
        {list.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {scope === "project" ? t.claudeMd.emptyProject : t.claudeMd.emptyGlobal}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {selected === "__generate__" ? (
          <div className="max-w-2xl space-y-3">
            <p style={{ color: "var(--text-muted)" }}>{t.claudeMd.generateIntro("/init")}</p>
            <AiAssistBox
              kind="claudemd"
              currentFields={{ content: "" }}
              onResult={createRootWithAi}
              forceOpen
              onCancel={() => setSelected(null)}
            />
          </div>
        ) : !selected ? (
          <div className="max-w-2xl space-y-3" style={{ color: "var(--text-muted)" }}>
            <p>{t.claudeMd.aboutP1}</p>
            {!hasRoot ? (
              <p>{t.claudeMd.aboutNoRoot(t.claudeMd.generateWithClaude, "/init")}</p>
            ) : (
              <p>{t.claudeMd.aboutPickOne}</p>
            )}
          </div>
        ) : (
          <div className="max-w-3xl space-y-3">
            <div className="text-sm font-mono" style={{ color: "var(--text-muted)" }}>
              {selected}
            </div>
            <AiAssistBox
              kind="claudemd"
              currentFields={{ content }}
              onResult={(data) => {
                if (typeof data.content === "string") setContent(data.content);
              }}
            />
            <MarkdownEditor value={content} onChange={setContent} minRows={20} showPreview />
            <div className="flex gap-2">
              <Button onClick={save}>{t.common.save}</Button>
              <Button variant="danger" onClick={() => remove(selected)}>
                {t.common.delete}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
