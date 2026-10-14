import type { TextareaHTMLAttributes } from "react";

interface Props extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  /** Para campos "crudos" (comandos, JSON) que se ven mejor en monoespaciada. */
  mono?: boolean;
  /** text-xs en vez de text-sm — campos compactos o de código. */
  small?: boolean;
}

/** Textarea estándar — mismo look que TextInput. Ver TextInput.tsx para por
 * qué `className` solo debería traer layout, no color/borde/padding/tamaño. */
export default function TextArea({ mono = false, small = false, className = "", style, ...props }: Props) {
  return (
    <textarea
      {...props}
      className={`px-3 py-1.5 rounded-lg border ${small ? "text-xs" : "text-sm"} ${mono ? "font-mono" : ""} ${className}`}
      style={{ borderColor: "var(--border)", ...style }}
    />
  );
}
