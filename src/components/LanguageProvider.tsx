import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { api } from "../api";
import { dictionaries, type Lang, type Strings } from "../strings";

interface LanguageApi {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: Strings;
}

const LanguageContext = createContext<LanguageApi | null>(null);

const STORAGE_KEY = "clman-lang";

function loadLang(): Lang {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "en" || saved === "es") return saved;
  } catch {
    // localStorage puede no estar disponible (ventana privada, etc.) — default abajo.
  }
  return "en";
}

export function useLanguage(): LanguageApi {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error("useLanguage tiene que usarse adentro de <LanguageProvider>");
  return ctx;
}

export default function LanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(loadLang);

  useEffect(() => {
    api.setLanguage(lang).catch(() => {
      // el backend arranca en inglés por default — si esta llamada falla
      // (ventana cerrándose, etc.) los mensajes de error siguen funcionando.
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setLang = (l: Lang) => {
    setLangState(l);
    try {
      localStorage.setItem(STORAGE_KEY, l);
    } catch {
      // per-viewer convenience nomás — si no se puede guardar, no pasa nada.
    }
    api.setLanguage(l).catch(() => {
      // ídem arriba.
    });
  };

  return (
    <LanguageContext.Provider value={{ lang, setLang, t: dictionaries[lang] }}>
      {children}
    </LanguageContext.Provider>
  );
}
