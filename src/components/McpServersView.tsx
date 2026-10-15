import { useEffect, useState } from "react";
import { api } from "../api";
import type { McpServerSummary, Scope } from "../types";
import AiAssistBox from "./AiAssistBox";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import Select from "./Select";
import TagInput from "./TagInput";
import TextArea from "./TextArea";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";

interface Props {
  scope: Scope;
  onListChanged: (list: McpServerSummary[]) => void;
  refreshSignal: number;
  /** Set by the Advisor module when it sends the user here with fields it
   * already generated from a described need — applied as if this view's
   * own AiAssistBox had just returned that same result. */
  prefill?: Record<string, unknown>;
  onPrefillConsumed?: () => void;
  /** Set by Global Search to jump straight to an existing server. */
  openId?: string;
  onOpenIdConsumed?: () => void;
}

const emptyForm = () => ({
  name: "",
  serverType: "stdio",
  command: "",
  args: [] as string[],
  env: "",
  url: "",
  headers: "",
});

type McpServerForm = ReturnType<typeof emptyForm>;

function mergeAiResult(base: McpServerForm, data: Record<string, unknown>): McpServerForm {
  return {
    name: typeof data.name === "string" ? data.name : base.name,
    serverType: typeof data.serverType === "string" ? data.serverType : base.serverType,
    command: typeof data.command === "string" ? data.command : base.command,
    args: Array.isArray(data.args) ? (data.args as string[]) : base.args,
    env: typeof data.env === "string" ? data.env : base.env,
    url: typeof data.url === "string" ? data.url : base.url,
    headers: typeof data.headers === "string" ? data.headers : base.headers,
  };
}

const isRemote = (serverType: string) => serverType === "http" || serverType === "sse";

export default function McpServersView({
  scope,
  onListChanged,
  refreshSignal,
  prefill,
  onPrefillConsumed,
  openId,
  onOpenIdConsumed,
}: Props) {
  const [list, setList] = useState<McpServerSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [originalName, setOriginalName] = useState<string | null>(null);
  const [form, setForm] = useState(emptyForm());
  const toast = useToast();
  const { t } = useLanguage();

  const refresh = async () => {
    const items = await api.listMcpServers(scope);
    setList(items);
    onListChanged(items);
  };

  useEffect(() => {
    refresh();
    setSelected(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  // Must run after the `[scope]` effect above — see RulesView for why.
  useEffect(() => {
    if (!prefill) return;
    setSelected("__new__");
    setOriginalName(null);
    setForm(mergeAiResult(emptyForm(), prefill));
    onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [prefill]);

  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  const openNew = () => {
    setSelected("__new__");
    setOriginalName(null);
    setForm(emptyForm());
  };

  const openExisting = async (name: string) => {
    const server = await api.readMcpServer(scope, name);
    setSelected(name);
    setOriginalName(name);
    setForm({
      name: server.name,
      serverType: server.server_type,
      command: server.command ?? "",
      args: server.args,
      env: server.env ?? "",
      url: server.url ?? "",
      headers: server.headers ?? "",
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
      const summary = await api.saveMcpServer({
        scope,
        originalName,
        name: form.name.trim(),
        serverType: form.serverType,
        command: form.command.trim() || null,
        args: form.args,
        env: form.env.trim() || null,
        url: form.url.trim() || null,
        headers: form.headers.trim() || null,
      });
      toast.success(t.common.saved);
      await refresh();
      setOriginalName(summary.name);
      setSelected(summary.name);
      setForm({ ...form, name: summary.name });
    } catch (e) {
      toast.error(String(e));
    }
  };

  const remove = async (name: string) => {
    if (!window.confirm(t.mcpServers.deleteConfirm(name))) return;
    await api.deleteMcpServer(scope, name);
    setSelected(null);
    await refresh();
  };

  const moveToOtherScope = async () => {
    if (!originalName) return;
    const otherScope: Scope = scope === "project" ? "global" : "project";
    try {
      await api.saveMcpServer({
        scope: otherScope,
        originalName: null,
        name: form.name.trim(),
        serverType: form.serverType,
        command: form.command.trim() || null,
        args: form.args,
        env: form.env.trim() || null,
        url: form.url.trim() || null,
        headers: form.headers.trim() || null,
      });
      await api.deleteMcpServer(scope, originalName);
      toast.success(
        t.common.movedToast(
          form.name || originalName,
          otherScope === "project" ? t.app.scopeProject : t.app.scopeGlobal,
        ),
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
        <div className="p-3 space-y-2">
          <Button full onClick={openNew}>
            {t.mcpServers.newServer}
          </Button>
        </div>
        {list.map((s) => (
          <ListRow
            key={s.name}
            active={selected === s.name}
            onClick={() => openExisting(s.name)}
            title={s.name}
            subtitle={s.server_type}
          />
        ))}
        {list.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {t.mcpServers.emptyList}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {!selected ? (
          <div className="max-w-2xl space-y-3" style={{ color: "var(--text-muted)" }}>
            <p>{t.mcpServers.emptyMainP1}</p>
            <p className="text-xs">{t.mcpServers.emptyMainP2(".mcp.json", "~/.claude.json")}</p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-4">
            <AiAssistBox
              kind="mcpServer"
              currentFields={form}
              onResult={(data) => setForm((prev) => mergeAiResult(prev, data))}
            />

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.mcpServers.nameLabel}
              </label>
              <TextInput
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder={t.mcpServers.namePlaceholder}
                className="w-full mt-1"
              />
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.mcpServers.serverTypeLabel}
              </label>
              <Select
                value={form.serverType}
                onChange={(e) => setForm({ ...form, serverType: e.target.value })}
                className="w-full mt-1"
              >
                <option value="stdio">stdio</option>
                <option value="http">http</option>
                <option value="sse">sse</option>
              </Select>
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.mcpServers.serverTypeHint}
              </div>
            </div>

            {isRemote(form.serverType) ? (
              <>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.mcpServers.urlLabel} <span style={{ color: "var(--danger)" }}>*</span>
                  </label>
                  <TextInput
                    value={form.url}
                    onChange={(e) => setForm({ ...form, url: e.target.value })}
                    placeholder={t.mcpServers.urlPlaceholder}
                    className="w-full mt-1"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.mcpServers.headersLabel}
                  </label>
                  <TextArea
                    value={form.headers}
                    onChange={(e) => setForm({ ...form, headers: e.target.value })}
                    rows={2}
                    placeholder={t.mcpServers.headersPlaceholder}
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
              </>
            ) : (
              <>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.mcpServers.commandLabel} <span style={{ color: "var(--danger)" }}>*</span>
                  </label>
                  <TextInput
                    value={form.command}
                    onChange={(e) => setForm({ ...form, command: e.target.value })}
                    placeholder={t.mcpServers.commandPlaceholder}
                    className="w-full mt-1"
                    mono
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.mcpServers.argsLabel}
                  </label>
                  <div className="mt-1">
                    <TagInput
                      tags={form.args}
                      onChange={(args) => setForm({ ...form, args })}
                      suggestions={[]}
                      placeholder={t.mcpServers.argsPlaceholder}
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.mcpServers.envLabel}
                  </label>
                  <TextArea
                    value={form.env}
                    onChange={(e) => setForm({ ...form, env: e.target.value })}
                    rows={2}
                    placeholder={t.mcpServers.envPlaceholder}
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
              </>
            )}

            <div className="flex gap-2 pt-2">
              <Button onClick={save}>{t.common.save}</Button>
              {originalName && (
                <>
                  <Button variant="ghost" onClick={moveToOtherScope}>
                    {scope === "project" ? t.common.moveToGlobal : t.common.moveToProject}
                  </Button>
                  <Button variant="danger" onClick={() => remove(originalName)}>
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
