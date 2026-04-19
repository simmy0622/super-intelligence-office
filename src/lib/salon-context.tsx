import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { listSalons, type Salon } from "./client";

const ACTIVE_SALON_KEY = "agent-salon:active-salon";
const GENERAL_SALON_ID = 1;

interface SalonContextValue {
  activeSalonId: number;
  setActiveSalonId: (id: number) => void;
  salons: Salon[];
  refreshSalons: () => Promise<void>;
}

const SalonContext = createContext<SalonContextValue | null>(null);

export function SalonProvider({ children }: { children: ReactNode }) {
  const [salons, setSalons] = useState<Salon[]>([]);
  const [activeSalonId, setActiveSalonIdState] = useState(() => {
    const raw = localStorage.getItem(ACTIVE_SALON_KEY);
    const parsed = raw ? Number.parseInt(raw, 10) : GENERAL_SALON_ID;
    return Number.isFinite(parsed) ? parsed : GENERAL_SALON_ID;
  });

  const setActiveSalonId = useCallback((id: number) => {
    setActiveSalonIdState(id);
    localStorage.setItem(ACTIVE_SALON_KEY, String(id));
  }, []);

  const refreshSalons = useCallback(async () => {
    const next = await listSalons();
    setSalons(next);
    if (next.length > 0 && !next.some((salon) => salon.id === activeSalonId)) {
      setActiveSalonId(next[0].id);
    }
  }, [activeSalonId, setActiveSalonId]);

  useEffect(() => {
    void refreshSalons().catch(() => {
      setSalons([]);
    });
  }, [refreshSalons]);

  const value = useMemo(
    () => ({ activeSalonId, setActiveSalonId, salons, refreshSalons }),
    [activeSalonId, setActiveSalonId, salons, refreshSalons],
  );

  return <SalonContext.Provider value={value}>{children}</SalonContext.Provider>;
}

export function useSalon() {
  const context = useContext(SalonContext);
  if (!context) {
    throw new Error("useSalon must be used within SalonProvider");
  }
  return context;
}
