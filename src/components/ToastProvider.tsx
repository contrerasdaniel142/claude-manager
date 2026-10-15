import { createContext, useCallback, useContext, useRef, useState, type ReactNode } from "react";

export interface ToastAction {
  label: string;
  onClick: () => void | Promise<void>;
  disabled?: boolean;
}

export interface ToastOptions {
  action?: ToastAction;
  sticky?: boolean;
}

interface Toast {
  id: number;
  kind: "success" | "error" | "info";
  message: string;
  action?: ToastAction;
  sticky?: boolean;
}

interface ToastApi {
  success: (message: string, options?: ToastOptions) => number;
  error: (message: string, options?: ToastOptions) => number;
  info: (message: string, options?: ToastOptions) => number;
  update: (id: number, message: string, options?: ToastOptions) => void;
  dismiss: (id: number) => void;
}

const ToastContext = createContext<ToastApi | null>(null);

/** Notificación flotante — reemplaza los Banner inline para feedback
 * puntual de una acción (guardar, borrar, generar con IA). Los Banner
 * siguen existiendo para avisos persistentes ligados al estado de los
 * datos (ej. "settings.json tiene entradas inválidas"). */
export function useToast(): ToastApi {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast tiene que usarse adentro de <ToastProvider>");
  return ctx;
}

const DURATION_MS: Record<Toast["kind"], number> = {
  success: 3500,
  info: 3500,
  error: 6000,
};

export default function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(0);

  const dismiss = useCallback((id: number) => {
    setToasts((ts) => ts.filter((t) => t.id !== id));
  }, []);

  const push = useCallback(
    (kind: Toast["kind"], message: string, options?: ToastOptions): number => {
      const id = nextId.current++;
      setToasts((ts) => [...ts, { id, kind, message, action: options?.action, sticky: options?.sticky }]);
      if (!options?.sticky) {
        setTimeout(() => dismiss(id), DURATION_MS[kind]);
      }
      return id;
    },
    [dismiss],
  );

  const update = useCallback(
    (id: number, message: string, options?: ToastOptions) => {
      setToasts((ts) =>
        ts.map((t) =>
          t.id === id
            ? { ...t, message, action: options?.action, sticky: options?.sticky }
            : t
        )
      );
    },
    [],
  );

  const api: ToastApi = {
    success: (m, opt) => push("success", m, opt),
    error: (m, opt) => push("error", m, opt),
    info: (m, opt) => push("info", m, opt),
    update,
    dismiss,
  };

  return (
    <ToastContext.Provider value={api}>
      {children}
      <div
        className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 items-end"
        style={{ pointerEvents: "none" }}
      >
        {toasts.map((t) => (
          <ToastItem key={t.id} toast={t} onDismiss={() => dismiss(t.id)} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({ toast, onDismiss }: { toast: Toast; onDismiss: () => void }) {
  const color =
    toast.kind === "error" ? "var(--danger)" : toast.kind === "success" ? "var(--ok)" : "var(--accent)";
  return (
    <div
      onClick={() => {
        if (!toast.action?.disabled) onDismiss();
      }}
      className="toast-enter rounded-xl px-4 py-3 text-sm max-w-sm whitespace-pre-line cursor-pointer flex flex-col gap-2"
      style={{
        background: "var(--bg-elevated)",
        color: "var(--text)",
        borderLeft: `3px solid ${color}`,
        boxShadow: "0 10px 30px rgba(0,0,0,0.2), 0 2px 6px rgba(0,0,0,0.12)",
        pointerEvents: "auto",
      }}
    >
      <div>{toast.message}</div>
      {toast.action && (
        <div className="flex justify-end pt-1">
          <button
            type="button"
            disabled={toast.action.disabled}
            onClick={async (e) => {
              e.stopPropagation();
              await toast.action?.onClick();
            }}
            className="px-2.5 py-1 text-xs rounded-md font-medium transition-opacity cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            style={{
              background: "var(--accent)",
              color: "var(--accent-fg, #ffffff)",
            }}
          >
            {toast.action.label}
          </button>
        </div>
      )}
    </div>
  );
}
