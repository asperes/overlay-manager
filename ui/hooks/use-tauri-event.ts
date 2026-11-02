import { useEffect, useEffectEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import type { EventPayloads } from "@/lib/tauri";

/** Subscribes to a backend event for the component's lifetime. */
export function useTauriEvent<K extends keyof EventPayloads>(event: K, handler: (payload: EventPayloads[K]) => void) {
  const onEvent = useEffectEvent(handler);

  useEffect(() => {
    const unlisten = listen<EventPayloads[K]>(event, (e) => onEvent(e.payload));
    return () => void unlisten.then((stop) => stop());
  }, [event]);
}
