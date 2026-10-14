import type { SelectHTMLAttributes } from "react";

interface Props extends SelectHTMLAttributes<HTMLSelectElement> {
  /** "compact": para selects chicos fuera de un formulario (ej. el de
   * idioma en el sidebar). Default: el tamaño estándar de un campo de
   * formulario. No se mezcla con `className` para no pisar padding/tamaño
   * con una utilidad de igual propiedad — ver TextInput.tsx. */
  compact?: boolean;
}

/** Select estándar — mismo look que TextInput. Ver TextInput.tsx para por
 * qué `className` solo debería traer layout, no color/borde/padding. */
export default function Select({ compact = false, className = "", style, children, ...props }: Props) {
  const sizeClass = compact ? "px-1.5 py-1 text-xs" : "px-3 py-1.5 text-sm";
  return (
    <select
      {...props}
      className={`${sizeClass} rounded-lg border ${className}`}
      style={{ borderColor: "var(--border)", ...style }}
    >
      {children}
    </select>
  );
}
