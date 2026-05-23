import { create } from "zustand";

interface ToastState {
  message: string;
  visible: boolean;
  timer: ReturnType<typeof setTimeout> | null;
  show: (message: string) => void;
}

export const useToastStore = create<ToastState>((set, get) => ({
  message: "",
  visible: false,
  timer: null,

  show: (message: string) => {
    const prev = get().timer;
    if (prev) clearTimeout(prev);

    set({ message, visible: true });

    const timer = setTimeout(() => {
      set({ visible: false });
    }, 1800);
    set({ timer });
  },
}));
