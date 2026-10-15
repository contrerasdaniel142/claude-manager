import type { ReactNode } from "react";

interface Props {
  active: boolean;
  onClick: () => void;
  title: ReactNode;
  subtitle?: ReactNode;
}

/** Fila clickeable de una lista con estado seleccionado — mismo look en el
 * panel izquierdo de CLAUDE.md, Rules, Agents, Skills y Hooks. */
export default function ListRow({ active, onClick, title, subtitle }: Props) {
  return (
    <button
      onClick={onClick}
      className="w-full text-left px-3 py-2 border-b cursor-pointer"
      style={{ borderColor: "var(--border)", background: active ? "var(--accent-soft)" : "transparent" }}
    >
      <div className="text-sm font-medium flex items-center gap-1.5 truncate">{title}</div>
      {subtitle && (
        <div className="text-xs truncate" style={{ color: "var(--text-muted)" }}>
          {subtitle}
        </div>
      )}
    </button>
  );
}
