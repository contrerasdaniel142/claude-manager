import { useLanguage } from "./LanguageProvider";
import Select from "./Select";

interface Props {
  label: string;
  value: "" | "true" | "false";
  onChange: (v: "" | "true" | "false") => void;
  hint?: string;
}

/** Tri-state: "" (unset, the key isn't written) / "true" / "false" — for
 * `Option<bool>` fields where "absent" and "false" are different things. */
export default function BoolSelect({ label, value, onChange, hint }: Props) {
  const { t } = useLanguage();
  return (
    <div>
      <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
        {label}
      </label>
      <Select value={value} onChange={(e) => onChange(e.target.value as "" | "true" | "false")} className="w-full mt-1">
        <option value="">{t.boolSelect.unset}</option>
        <option value="true">{t.boolSelect.yes}</option>
        <option value="false">{t.boolSelect.no}</option>
      </Select>
      {hint && (
        <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
          {hint}
        </div>
      )}
    </div>
  );
}
