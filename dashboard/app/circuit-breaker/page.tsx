"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { AlertTriangle, Activity, CheckCircle, XCircle, RefreshCw, Power, PowerOff, TrendingUp } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface CircuitBreakerAnalytics {
  total_policies: number;
  open_circuits: number;
  closed_circuits: number;
  half_open_circuits: number;
  recent_transitions: Array<{
    policy_id: string;
    policy_type: string;
    state_transition: string;
    error_rate: number;
    error_count: number;
    total_requests: number;
    triggered_by: string;
    notes: string | null;
    timestamp: string;
  }>;
  policies: Array<{
    policy_id: string;
    policy_type: string;
    enabled: boolean;
    current_state: string;
    error_threshold: number;
    current_error_rate: number;
    error_count: number;
    total_requests: number;
    opened_at: string | null;
    last_error_at: string | null;
    cooldown_minutes: number;
  }>;
}

async function fetchCircuitBreakerAnalytics(): Promise<CircuitBreakerAnalytics> {
  const res = await fetch(`${API_BASE}/analytics/circuit-breaker`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch circuit breaker analytics: ${res.status}`);
  }
  return res.json();
}

async function configureCircuitBreaker(policyId: string, enabled: boolean, errorThreshold?: number, windowMinutes?: number, cooldownMinutes?: number) {
  const res = await fetch(`${API_BASE}/policies/${policyId}/circuit-breaker/config`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      enabled,
      error_threshold: errorThreshold,
      window_minutes: windowMinutes,
      cooldown_minutes: cooldownMinutes,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    const error = await res.json();
    throw new Error(error.message || `Failed to configure circuit breaker: ${res.status}`);
  }
  return res.json();
}

export default function CircuitBreakerPage() {
  const queryClient = useQueryClient();
  const [selectedPolicy, setSelectedPolicy] = useState<string | null>(null);

  const { data: analytics, isLoading, error, refetch } = useQuery({
    queryKey: ["circuit-breaker-analytics"],
    queryFn: fetchCircuitBreakerAnalytics,
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const configureMutation = useMutation({
    mutationFn: ({ policyId, enabled, errorThreshold, windowMinutes, cooldownMinutes }: {
      policyId: string;
      enabled: boolean;
      errorThreshold?: number;
      windowMinutes?: number;
      cooldownMinutes?: number;
    }) => configureCircuitBreaker(policyId, enabled, errorThreshold, windowMinutes, cooldownMinutes),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["circuit-breaker-analytics"] });
    },
  });

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="animate-pulse space-y-4">
            <div className="h-8 bg-slate-800 rounded w-1/3"></div>
            <div className="h-64 bg-slate-800 rounded"></div>
          </div>
        </div>
      </DashboardLayout>
    );
  }

  if (error) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="bg-red-900/20 border border-red-800 rounded-lg p-4 text-red-400">
            <AlertTriangle className="inline mr-2" size={20} />
            Error: {error instanceof Error ? error.message : "Unknown error"}
          </div>
        </div>
      </DashboardLayout>
    );
  }

  if (!analytics) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="text-slate-400">No circuit breaker data available</div>
        </div>
      </DashboardLayout>
    );
  }

  const getStateColor = (state: string) => {
    switch (state) {
      case "OPEN":
        return "bg-red-900/30 text-red-400 border-red-800";
      case "HALF_OPEN":
        return "bg-yellow-900/30 text-yellow-400 border-yellow-800";
      case "CLOSED":
        return "bg-emerald-900/30 text-emerald-400 border-emerald-800";
      default:
        return "bg-slate-700 text-slate-400";
    }
  };

  const getStateIcon = (state: string) => {
    switch (state) {
      case "OPEN":
        return <XCircle size={20} />;
      case "HALF_OPEN":
        return <RefreshCw size={20} className="animate-spin" />;
      case "CLOSED":
        return <CheckCircle size={20} />;
      default:
        return <Activity size={20} />;
    }
  };

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
              <Activity className="text-emerald-400" size={32} />
              Circuit Breaker Dashboard
            </h1>
            <p className="text-slate-400 mt-2">
              Monitor and manage circuit breakers to prevent policy errors from affecting production
            </p>
          </div>
          <button
            onClick={() => refetch()}
            className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200 hover:bg-slate-700 flex items-center gap-2"
          >
            <RefreshCw size={16} />
            Refresh
          </button>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Total Policies</span>
              <Activity className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{analytics.total_policies}</div>
            <div className="text-xs text-slate-500 mt-1">With circuit breaker enabled</div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">Open Circuits</span>
              <XCircle className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{analytics.open_circuits}</div>
            <div className="text-xs text-red-500 mt-1">Policies disabled</div>
          </div>

          <div className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-emerald-400 text-sm">Closed Circuits</span>
              <CheckCircle className="text-emerald-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-emerald-400">{analytics.closed_circuits}</div>
            <div className="text-xs text-emerald-500 mt-1">Policies active</div>
          </div>

          <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-yellow-400 text-sm">Half-Open</span>
              <RefreshCw className="text-yellow-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-yellow-400">{analytics.half_open_circuits}</div>
            <div className="text-xs text-yellow-500 mt-1">Testing recovery</div>
          </div>
        </div>

        {/* Policies List */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Power className="text-emerald-400" size={20} />
            Policy Status
          </h2>
          {analytics.policies.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No policies with circuit breaker enabled</div>
          ) : (
            <div className="space-y-3">
              {analytics.policies.map((policy) => (
                <div key={policy.policy_id} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-3">
                      {getStateIcon(policy.current_state)}
                      <div>
                        <span className="font-semibold text-slate-200">{policy.policy_type}</span>
                        <span className="text-xs text-slate-400 ml-2 font-mono">{policy.policy_id.slice(0, 8)}</span>
                      </div>
                    </div>
                    <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${getStateColor(policy.current_state)}`}>
                      {policy.current_state}
                    </span>
                  </div>
                  
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-3">
                    <div>
                      <span className="text-slate-400">Error Rate:</span>
                      <div className={`font-semibold ${policy.current_error_rate > policy.error_threshold ? "text-red-400" : "text-slate-200"}`}>
                        {policy.current_error_rate.toFixed(2)}% / {policy.error_threshold}%
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Errors:</span>
                      <div className="font-semibold text-slate-200">{policy.error_count} / {policy.total_requests}</div>
                    </div>
                    <div>
                      <span className="text-slate-400">Cooldown:</span>
                      <div className="font-semibold text-slate-200">{policy.cooldown_minutes} min</div>
                    </div>
                    {policy.opened_at && (
                      <div>
                        <span className="text-slate-400">Opened:</span>
                        <div className="font-semibold text-slate-200 text-xs">
                          {format(new Date(policy.opened_at), "MMM d, HH:mm")}
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="h-2 bg-slate-700 rounded-full overflow-hidden mb-3">
                    <div
                      className={`h-full ${
                        policy.current_error_rate > policy.error_threshold
                          ? "bg-red-500"
                          : policy.current_error_rate > policy.error_threshold * 0.7
                          ? "bg-yellow-500"
                          : "bg-emerald-500"
                      }`}
                      style={{ width: `${Math.min((policy.current_error_rate / policy.error_threshold) * 100, 100)}%` }}
                    />
                  </div>

                  <div className="flex gap-2">
                    <button
                      onClick={() => {
                        setSelectedPolicy(policy.policy_id);
                        configureMutation.mutate({
                          policyId: policy.policy_id,
                          enabled: !policy.enabled,
                        });
                      }}
                      className={`px-3 py-1 rounded text-xs font-semibold ${
                        policy.enabled
                          ? "bg-red-900/30 text-red-400 border border-red-800 hover:bg-red-900/50"
                          : "bg-emerald-900/30 text-emerald-400 border border-emerald-800 hover:bg-emerald-900/50"
                      }`}
                    >
                      {policy.enabled ? <><PowerOff size={14} className="inline mr-1" /> Disable</> : <><Power size={14} className="inline mr-1" /> Enable</>}
                    </button>
                    {policy.current_state === "OPEN" && (
                      <button
                        onClick={() => {
                          configureMutation.mutate({
                            policyId: policy.policy_id,
                            enabled: true,
                          });
                        }}
                        className="px-3 py-1 rounded text-xs font-semibold bg-yellow-900/30 text-yellow-400 border border-yellow-800 hover:bg-yellow-900/50"
                      >
                        <RefreshCw size={14} className="inline mr-1" />
                        Reset to Half-Open
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Recent Transitions */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <TrendingUp className="text-emerald-400" size={20} />
            Recent Transitions
          </h2>
          {analytics.recent_transitions.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No recent circuit breaker transitions</div>
          ) : (
            <div className="space-y-2">
              {analytics.recent_transitions.slice(0, 10).map((transition, idx) => (
                <div key={idx} className="bg-slate-900/50 rounded-lg p-3 border border-slate-700">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      {getStateIcon(transition.state_transition)}
                      <div>
                        <span className="font-semibold text-slate-200">{transition.policy_type}</span>
                        <span className="text-xs text-slate-400 ml-2">{transition.state_transition}</span>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm text-slate-300">
                        {format(new Date(transition.timestamp), "MMM d, HH:mm:ss")}
                      </div>
                      <div className="text-xs text-slate-400">
                        Error: {transition.error_rate.toFixed(2)}% ({transition.error_count}/{transition.total_requests})
                      </div>
                    </div>
                  </div>
                  {transition.notes && (
                    <div className="text-xs text-slate-400 mt-2">{transition.notes}</div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

