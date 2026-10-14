import type { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** "primary" (default): fondo sólido accent, para la acción principal.
   * "danger": borde/texto danger, para Eliminar.
   * "ghost": sin borde, texto muted — para Cancelar y acciones secundarias.
   * "chip": pill chica, togglea con `active` — para selecciones tipo tag. */
  variant?: "primary" | "danger" | "ghost" | "chip";
  /** "md" (default): el tamaño de las acciones principales (Guardar, + Nuevo X).
   * "sm": el de las acciones secundarias (Generar, Cancelar, chips). */
  size?: "md" | "sm";
  /** Ocupa todo el ancho disponible — los "+ Nuevo X" del panel izquierdo. */
  full?: boolean;
  /** Solo para variant="chip": si está marcado/seleccionado. */
  active?: boolean;
}

/** Botón estándar — mismo look en toda la app. `className` solo debería
 * traer layout (w-*, shrink-0, etc), no color/padding: ver TextInput.tsx
 * para por qué una utilidad de Tailwind con la misma propiedad puede no
 * ganarle al estilo del componente según el orden del className. */
export default function Button({
  variant = "primary",
  size = "md",
  full = false,
  active = false,
  className = "",
  style,
  ...props
}: Props) {
  const sizeClass = size === "sm" ? "px-3 py-1 text-xs" : "px-4 py-1.5 text-sm";
  const shapeClass = variant === "chip" ? "rounded-full" : "rounded-lg";
  const base = `${shapeClass} font-medium ${sizeClass} ${full ? "w-full" : ""}`;

  const variantStyle: React.CSSProperties =
    variant === "primary"
      ? { background: "var(--accent)", color: "white" }
      : variant === "danger"
        ? { color: "var(--danger)", border: "1px solid var(--danger)" }
        : variant === "chip"
          ? active
            ? { background: "var(--accent)", color: "white" }
            : { border: "1px solid var(--border)", color: "var(--text)" }
          : { color: "var(--text-muted)" };

  return <button {...props} className={`${base} ${className}`} style={{ ...variantStyle, ...style }} />;
}
