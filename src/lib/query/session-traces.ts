import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { sessionTracesApi } from "@/lib/api/session-traces";
import type {
  SessionTraceSettings,
  TraceSessionDetailRequest,
  TraceSessionFilters,
} from "@/types/session-traces";

export const sessionTraceKeys = {
  settings: ["sessionTraceSettings"] as const,
  sessions: (filters?: TraceSessionFilters) =>
    ["traceSessions", filters ?? {}] as const,
  detail: (request?: TraceSessionDetailRequest) =>
    ["traceSessionDetail", request ?? {}] as const,
};

export function useSessionTraceSettingsQuery() {
  return useQuery({
    queryKey: sessionTraceKeys.settings,
    queryFn: sessionTracesApi.getSettings,
    staleTime: 30 * 1000,
  });
}

export function useUpdateSessionTraceSettingsMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: SessionTraceSettings) =>
      sessionTracesApi.setSettings(settings),
    onSuccess: (settings) => {
      queryClient.setQueryData(sessionTraceKeys.settings, settings);
      void queryClient.invalidateQueries({ queryKey: ["traceSessions"] });
    },
  });
}

export function usePruneSessionTracesMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: sessionTracesApi.prune,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["traceSessions"] });
      void queryClient.invalidateQueries({ queryKey: ["traceSessionDetail"] });
    },
  });
}

export function useTraceSessionsQuery(filters?: TraceSessionFilters) {
  return useQuery({
    queryKey: sessionTraceKeys.sessions(filters),
    queryFn: () => sessionTracesApi.listSessions(filters),
    staleTime: 30 * 1000,
  });
}

export function useTraceSessionDetailQuery(
  request?: TraceSessionDetailRequest,
) {
  return useQuery({
    queryKey: sessionTraceKeys.detail(request),
    queryFn: () => sessionTracesApi.getSessionDetail(request!),
    enabled: Boolean(request?.sessionId),
    staleTime: 15 * 1000,
  });
}
