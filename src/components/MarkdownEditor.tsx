import { useState } from "react";
import ReactMarkdown from "react-markdown";
import { useLanguage } from "./LanguageProvider";

interface Props {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  minRows?: number;
  showPreview?: boolean;
}

export default function MarkdownEditor({
  value,
  onChange,
  placeholder,
  minRows = 10,
  showPreview = false,
}: Props) {
  const [tab, setTab] = useState<"write" | "preview">("write");
  const { t } = useLanguage();

  return (
    <div className="rounded-lg border" style={{ borderColor: "var(--border)" }}>
      {showPreview && (
        <div className="flex text-xs border-b" style={{ borderColor: "var(--border)" }}>
          {(["write", "preview"] as const).map((tabKey) => (
            <button
              key={tabKey}
              onClick={() => setTab(tabKey)}
              className="px-3 py-1.5 capitalize cursor-pointer"
              style={{
                borderBottom: tab === tabKey ? "2px solid var(--accent)" : "2px solid transparent",
                color: tab === tabKey ? "var(--text)" : "var(--text-muted)",
              }}
            >
              {tabKey === "write" ? t.markdownEditor.write : t.markdownEditor.preview}
            </button>
          ))}
        </div>
      )}
      {tab === "write" || !showPreview ? (
        <textarea
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          rows={minRows}
          className="w-full p-3 text-sm font-mono outline-none resize-y"
          style={{ color: "var(--text)" }}
        />
      ) : (
        <div
          className="prose prose-sm max-w-none p-3 overflow-auto"
          style={{ color: "var(--text)", minHeight: `${minRows * 1.4}em` }}
        >
          <ReactMarkdown>{value || t.markdownEditor.emptyPlaceholder}</ReactMarkdown>
        </div>
      )}
    </div>
  );
}
