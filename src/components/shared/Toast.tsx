import { useState, useEffect, useCallback, createContext, useContext } from "react";
import { X, CheckCircle, AlertTriangle, XCircle, Info } from "lucide-react";

type ToastType = "success" | "error" | "warning" | "info";

interface Toast {
  id: number;
  type: ToastType;
  message: string;
}

interface ToastContextValue {
  addToast: (type: ToastType, message: string) => void;
}

const ToastContext = createContext<ToastContextValue>({
  addToast: () => {},
});

export function useToast() {
  return useContext(ToastContext);
}

let nextId = 0;

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((type: ToastType, message: string) => {
    const id = nextId++;
    setToasts((prev) => [...prev, { id, type, message }]);
  }, []);

  const removeToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ addToast }}>
      {children}
      <div className="fixed bottom-4 right-4 z-50 space-y-2 max-w-sm">
        {toasts.map((toast) => (
          <ToastItem
            key={toast.id}
            toast={toast}
            onDismiss={() => removeToast(toast.id)}
          />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({
  toast,
  onDismiss,
}: {
  toast: Toast;
  onDismiss: () => void;
}) {
  useEffect(() => {
    const timer = setTimeout(onDismiss, 5000);
    return () => clearTimeout(timer);
  }, [onDismiss]);

  const icon = {
    success: <CheckCircle size={16} className="text-green-400 shrink-0" />,
    error: <XCircle size={16} className="text-red-400 shrink-0" />,
    warning: <AlertTriangle size={16} className="text-amber-400 shrink-0" />,
    info: <Info size={16} className="text-blue-400 shrink-0" />,
  }[toast.type];

  const border = {
    success: "border-green-500/30",
    error: "border-red-500/30",
    warning: "border-amber-500/30",
    info: "border-blue-500/30",
  }[toast.type];

  return (
    <div
      className={`flex items-start gap-2 bg-zinc-800 border ${border} rounded-lg px-3 py-2.5 shadow-lg`}
    >
      {icon}
      <p className="text-sm text-zinc-200 flex-1">{toast.message}</p>
      <button
        onClick={onDismiss}
        className="p-0.5 text-zinc-500 hover:text-zinc-300 shrink-0"
      >
        <X size={12} />
      </button>
    </div>
  );
}
