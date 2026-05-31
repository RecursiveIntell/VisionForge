import { useConfigContext } from "../context/ConfigContext";

export function useConfig() {
  return useConfigContext();
}
