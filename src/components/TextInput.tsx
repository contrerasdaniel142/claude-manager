import type { InputHTMLAttributes, Ref } from "react";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  /** Para campos "crudos" (comandos, JSON) que se ven mejor en monoespaciada. */
  mono?: boolean;
  /** text-xs en vez de text-sm — campos compactos o de código. */
  small?: boolean;
  ref?: Ref<HTMLInputElement>;
}

/** Input de texto estándar — mismo look en toda la app. `className` solo
 * debería traer layout (w-*, mt-*, etc), no color/borde/padding/tamaño de
 * texto: esos ya los pone el componente (con `mono`/`small` para las
 * variantes que existen), y una clase de Tailwind con la misma propiedad
 * puede perder el override según el orden en que Tailwind genera el CSS,
 * no el orden en el string de className. */
export default function TextInput({ mono = false, small = false, className = "", style, ref, ...props }: Props) {
  return (
    <input
      {...props}
      ref={ref}
      className={`px-3 py-1.5 rounded-lg border ${small ? "text-xs" : "text-sm"} ${mono ? "font-mono" : ""} ${className}`}
      style={{ borderColor: "var(--border)", ...style }}
    />
  );
}
