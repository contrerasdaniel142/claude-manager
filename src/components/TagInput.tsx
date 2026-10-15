import { useState } from "react";
import { useLanguage } from "./LanguageProvider";

interface Props {
  tags: string[];
  onChange: (tags: string[]) => void;
  suggestions: string[];
  placeholder?: string;
}

export default function TagInput({ tags, onChange, suggestions, placeholder }: Props) {
  const [draft, setDraft] = useState("");
  const [open, setOpen] = useState(false);
  const { t } = useLanguage();

  const add = (raw: string) => {
    const v = raw.trim();
    if (!v || tags.includes(v)) return;
    onChange([...tags, v]);
    setDraft("");
  };

  const remove = (v: string) => onChange(tags.filter((t) => t !== v));

  const filtered = suggestions.filter(
    (s) => !tags.includes(s) && s.toLowerCase().includes(draft.toLowerCase()),
  );

  return (
    <div className="relative">
      <div
        className="flex flex-wrap gap-1.5 p-1.5 rounded-lg border min-h-[38px]"
        style={{ borderColor: "var(--border)" }}
      >
        {tags.map((tag) => (
          <span
            key={tag}
            className="inline-flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-full text-xs"
            style={{ background: "var(--accent-soft)", color: "var(--accent)" }}
          >
            {tag}
            <button
              onClick={() => remove(tag)}
              className="rounded-full w-4 h-4 inline-flex items-center justify-center hover:opacity-70 cursor-pointer"
              aria-label={t.tagInput.remove(tag)}
            >
              ×
            </button>
          </span>
        ))}
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onFocus={() => setOpen(true)}
          onBlur={() => setTimeout(() => setOpen(false), 120)}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === ",") {
              e.preventDefault();
              add(draft);
            } else if (e.key === "Backspace" && draft === "" && tags.length > 0) {
              remove(tags[tags.length - 1]);
            }
          }}
          placeholder={tags.length === 0 ? placeholder : ""}
          className="flex-1 min-w-[120px] outline-none text-sm px-1"
          style={{ background: "transparent", color: "var(--text)" }}
        />
      </div>
      {open && filtered.length > 0 && (
        <div
          className="absolute z-10 mt-1 w-full max-h-48 overflow-auto rounded-lg border shadow-lg"
          style={{ background: "var(--bg-elevated)", borderColor: "var(--border)" }}
        >
          {filtered.map((s) => (
            <button
              key={s}
              onMouseDown={(e) => {
                e.preventDefault();
                add(s);
              }}
              className="w-full text-left px-3 py-1.5 text-sm hover:opacity-80 cursor-pointer"
              style={{ color: "var(--text)" }}
            >
              {s}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
