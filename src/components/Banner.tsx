import type { ReactNode } from "react";

interface Props {
  kind: "error" | "success" | "info";
  children: ReactNode;
}

export default function Banner({ kind, children }: Props) {
  const color = kind === "error" ? "var(--danger)" : kind === "success" ? "var(--ok)" : "var(--accent)";
  return (
    <div
      className="rounded-lg px-3 py-2 text-sm mb-3 whitespace-pre-line"
      style={{ background: `${color}1a`, color, border: `1px solid ${color}55` }}
    >
      {children}
    </div>
  );
}
