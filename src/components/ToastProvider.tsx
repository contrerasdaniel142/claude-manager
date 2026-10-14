import { createContext, useCallback, useContext, useRef, useState, type ReactNode } from "react";

interface Toast {
  id: number;
  kind: "success" | "error" | "info";
  message: string;
}

interface ToastApi {
  success: (message: string) => void;
  error: (message: string) => void;
  info: (message: string) => void;
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
    (kind: Toast["kind"], message: string) => {
      const id = nextId.current++;
      setToasts((ts) => [...ts, { id, kind, message }]);
      setTimeout(() => dismiss(id), DURATION_MS[kind]);
    },
    [dismiss],
  );

  const api: ToastApi = {
    success: (m) => push("success", m),
    error: (m) => push("error", m),
    info: (m) => push("info", m),
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
      onClick={onDismiss}
      className="toast-enter rounded-xl px-4 py-3 text-sm max-w-sm whitespace-pre-line cursor-pointer"
      style={{
        background: "var(--bg-elevated)",
        color: "var(--text)",
        borderLeft: `3px solid ${color}`,
        boxShadow: "0 10px 30px rgba(0,0,0,0.2), 0 2px 6px rgba(0,0,0,0.12)",
        pointerEvents: "auto",
      }}
    >
      {toast.message}
    </div>
  );
}
