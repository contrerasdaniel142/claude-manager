import type { InputHTMLAttributes } from "react";

interface Props extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label: string;
}

/** Checkbox con su label al lado — mismo look en toda la app. */
export default function Checkbox({ label, className = "", ...props }: Props) {
  return (
    <label
      className={`flex items-center gap-2 text-sm pb-1.5 ${className}`}
      style={{ color: "var(--text-muted)" }}
    >
      <input type="checkbox" {...props} />
      {label}
    </label>
  );
}
