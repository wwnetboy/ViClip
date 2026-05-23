import { useCallback, useRef, useState } from "react";

export function useHoverSwitch(
  onSwitch: (key: string) => void,
  delayMs: number = 2000
) {
  const onSwitchRef = useRef(onSwitch);
  onSwitchRef.current = onSwitch;

  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [progressKey, setProgressKey] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);

  const clearTimers = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const handleEnter = useCallback(
    (key: string) => {
      clearTimers();
      setProgressKey(key);
      setProgress(0);

      const startTime = Date.now();
      intervalRef.current = setInterval(() => {
        const elapsed = Date.now() - startTime;
        setProgress(Math.min(elapsed / delayMs, 1));
      }, 50);

      timerRef.current = setTimeout(() => {
        clearTimers();
        onSwitchRef.current(key);
        setProgressKey(null);
        setProgress(0);
      }, delayMs);
    },
    [delayMs, clearTimers]
  );

  const handleLeave = useCallback(() => {
    clearTimers();
    setProgressKey(null);
    setProgress(0);
  }, [clearTimers]);

  return { progressKey, progress, handleEnter, handleLeave };
}
