"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { AlertTriangle, Activity, CheckCircle, XCircle, RefreshCw, Power, PowerOff, TrendingUp, History, Gauge, Clock } from "lucide-react";
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

interface CircuitBreakerHistory {
  transitions: Array<{
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
  total_count: number;
}

interface CircuitBreakerMetrics {
  policy_id: string;
  policy_type: string;
  error_rate_trends: Array<{
    timestamp: string;
    error_rate: number;
    error_count: number;
    total_requests: number;
  }>;
  recovery_times: Array<{
    opened_at: string;
    closed_at: string | null;
    recovery_time_minutes: number | null;
    state_transition: string;
  }>;
  average_recovery_time_minutes: number;
  total_open_events: number;
  total_close_events: number;
  current_state: string;
  time_in_current_state_minutes: number;
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

async function fetchCircuitBreakerHistory(policyId: string): Promise<CircuitBreakerHistory> {
  const res = await fetch(`${API_BASE}/policies/${policyId}/circuit-breaker/history?limit=100`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch history: ${res.status}`);
  }
  return res.json();
}

async function fetchCircuitBreakerMetrics(policyId: string): Promise<CircuitBreakerMetrics> {
  const res = await fetch(`${API_BASE}/policies/${policyId}/circuit-breaker/metrics?days=7`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch metrics: ${res.status}`);
  }
  return res.json();
}

async function configureCircuitBreaker(policyId: string, enabled: boolean, errorThreshold?: number, windowMinutes?: number, cooldownMinutes?: number) {
  const res = await fetch(`${API_BASE}/policies/${policyId}/circuit-breaker/config`, {
    method: "POST",
    headers: {
      ...getAuthHeaders(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      enabled,
      error_threshold: errorThreshold,
      window_minutes: windowMinutes,
      cooldown_minutes: cooldownMinutes,
    }),
  });
  if (!res.ok) {
    throw new Error(`Failed to configure: ${res.status}`);
  }
  return res.json();
}

async function controlCircuitBreaker(policyId: string, action: "OPEN" | "CLOSE" | "AUTO", reason?: string) {
  const res = await fetch(`${API_BASE}/policies/${policyId}/circuit-breaker/control`, {
    method: "POST",
    headers: {
      ...getAuthHeaders(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      action,
      reason,
    }),
  });
  if (!res.ok) {
    throw new Error(`Failed to control circuit breaker: ${res.status}`);
  }
  return res.json();
}

export default function CircuitBreakerPage() {
  const queryClient = useQueryClient();
  const [selectedPolicy, setSelectedPolicy] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"overview" | "history" | "metrics">("overview");

  const { data: analytics, isLoading, error, refetch } = useQuery({
    queryKey: ["circuit-breaker-analytics"],
    queryFn: fetchCircuitBreakerAnalytics,
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const { data: history, isLoading: historyLoading } = useQuery({
    queryKey: ["circuit-breaker-history", selectedPolicy],
    queryFn: () => selectedPolicy ? fetchCircuitBreakerHistory(selectedPolicy) : null,
    enabled: !!selectedPolicy && activeTab === "history",
  });

  const { data: metrics, isLoading: metricsLoading } = useQuery({
    queryKey: ["circuit-breaker-metrics", selectedPolicy],
    queryFn: () => selectedPolicy ? fetchCircuitBreakerMetrics(selectedPolicy) : null,
    enabled: !!selectedPolicy && activeTab === "metrics",
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

  const controlMutation = useMutation({
    mutationFn: ({ policyId, action, reason }: {
      policyId: string;
      action: "OPEN" | "CLOSE" | "AUTO";
      reason?: string;
    }) => controlCircuitBreaker(policyId, action, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["circuit-breaker-analytics"] });
      queryClient.invalidateQueries({ queryKey: ["circuit-breaker-history", selectedPolicy] });
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

  const currentPolicy = selectedPolicy ? analytics.policies.find(p => p.policy_id === selectedPolicy) : null;

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

        {/* Tabs */}
        <div className="flex gap-2 border-b border-slate-700">
          <button
            onClick={() => setActiveTab("overview")}
            className={`px-4 py-2 font-semibold border-b-2 transition-colors ${
              activeTab === "overview"
                ? "border-emerald-400 text-emerald-400"
                : "border-transparent text-slate-400 hover:text-slate-200"
            }`}
          >
            <Activity size={16} className="inline mr-2" />
            Overview
          </button>
          {selectedPolicy && (
            <>
              <button
                onClick={() => setActiveTab("history")}
                className={`px-4 py-2 font-semibold border-b-2 transition-colors ${
                  activeTab === "history"
                    ? "border-emerald-400 text-emerald-400"
                    : "border-transparent text-slate-400 hover:text-slate-200"
                }`}
              >
                <History size={16} className="inline mr-2" />
                History
              </button>
              <button
                onClick={() => setActiveTab("metrics")}
                className={`px-4 py-2 font-semibold border-b-2 transition-colors ${
                  activeTab === "metrics"
                    ? "border-emerald-400 text-emerald-400"
                    : "border-transparent text-slate-400 hover:text-slate-200"
                }`}
              >
                <Gauge size={16} className="inline mr-2" />
                Metrics
              </button>
            </>
          )}
        </div>

        {/* Content based on active tab */}
        {activeTab === "overview" && (
          <>
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

                      <div className="flex gap-2 flex-wrap">
                        <button
                          onClick={() => {
                            setSelectedPolicy(policy.policy_id);
                            setActiveTab("overview");
                          }}
                          className={`px-3 py-1 rounded text-xs font-semibold ${
                            selectedPolicy === policy.policy_id
                              ? "bg-emerald-900/50 text-emerald-400 border border-emerald-800"
                              : "bg-slate-700 text-slate-300 border border-slate-600 hover:bg-slate-600"
                          }`}
                        >
                          View Details
                        </button>
                        <button
                          onClick={() => {
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
                          <>
                            <button
                              onClick={() => {
                                controlMutation.mutate({
                                  policyId: policy.policy_id,
                                  action: "CLOSE",
                                  reason: "Manual close by operator",
                                });
                              }}
                              className="px-3 py-1 rounded text-xs font-semibold bg-emerald-900/30 text-emerald-400 border border-emerald-800 hover:bg-emerald-900/50"
                            >
                              <CheckCircle size={14} className="inline mr-1" />
                              Force Close
                            </button>
                            <button
                              onClick={() => {
                                controlMutation.mutate({
                                  policyId: policy.policy_id,
                                  action: "AUTO",
                                  reason: "Reset to auto mode",
                                });
                              }}
                              className="px-3 py-1 rounded text-xs font-semibold bg-blue-900/30 text-blue-400 border border-blue-800 hover:bg-blue-900/50"
                            >
                              <RefreshCw size={14} className="inline mr-1" />
                              Reset to Auto
                            </button>
                          </>
                        )}
                        {policy.current_state === "CLOSED" && (
                          <button
                            onClick={() => {
                              controlMutation.mutate({
                                policyId: policy.policy_id,
                                action: "OPEN",
                                reason: "Manual open by operator",
                              });
                            }}
                            className="px-3 py-1 rounded text-xs font-semibold bg-red-900/30 text-red-400 border border-red-800 hover:bg-red-900/50"
                          >
                            <XCircle size={14} className="inline mr-1" />
                            Force Open
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
          </>
        )}

        {activeTab === "history" && selectedPolicy && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <History className="text-emerald-400" size={20} />
              Circuit Breaker History
              {currentPolicy && (
                <span className="text-sm text-slate-400 font-normal ml-2">
                  - {currentPolicy.policy_type}
                </span>
              )}
            </h2>
            {historyLoading ? (
              <div className="text-slate-400 text-center py-8">Loading history...</div>
            ) : history && history.transitions.length > 0 ? (
              <div className="space-y-2">
                <div className="text-sm text-slate-400 mb-4">
                  Total transitions: {history.total_count}
                </div>
                {history.transitions.map((transition, idx) => (
                  <div key={idx} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-3">
                        {getStateIcon(transition.state_transition)}
                        <span className="font-semibold text-slate-200">{transition.state_transition}</span>
                        <span className="text-xs text-slate-400">by {transition.triggered_by}</span>
                      </div>
                      <div className="text-sm text-slate-300">
                        {format(new Date(transition.timestamp), "MMM d, yyyy HH:mm:ss")}
                      </div>
                    </div>
                    <div className="grid grid-cols-3 gap-4 text-sm text-slate-400">
                      <div>Error Rate: <span className="text-slate-200 font-semibold">{transition.error_rate.toFixed(2)}%</span></div>
                      <div>Errors: <span className="text-slate-200 font-semibold">{transition.error_count} / {transition.total_requests}</span></div>
                      <div>Policy: <span className="text-slate-200 font-semibold">{transition.policy_type}</span></div>
                    </div>
                    {transition.notes && (
                      <div className="text-xs text-slate-400 mt-2 italic">{transition.notes}</div>
                    )}
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-slate-400 text-center py-8">No history available for this policy</div>
            )}
          </div>
        )}

        {activeTab === "metrics" && selectedPolicy && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 space-y-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <Gauge className="text-emerald-400" size={20} />
              Circuit Breaker Metrics
              {currentPolicy && (
                <span className="text-sm text-slate-400 font-normal ml-2">
                  - {currentPolicy.policy_type}
                </span>
              )}
            </h2>
            {metricsLoading ? (
              <div className="text-slate-400 text-center py-8">Loading metrics...</div>
            ) : metrics ? (
              <>
                {/* Summary Cards */}
                <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                  <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <div className="text-slate-400 text-sm mb-1">Current State</div>
                    <div className={`text-2xl font-bold ${getStateColor(metrics.current_state).split(" ")[1]}`}>
                      {metrics.current_state}
                    </div>
                    <div className="text-xs text-slate-500 mt-1">
                      {metrics.time_in_current_state_minutes > 0
                        ? `${Math.round(metrics.time_in_current_state_minutes)} minutes`
                        : "Just changed"}
                    </div>
                  </div>
                  <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <div className="text-slate-400 text-sm mb-1">Avg Recovery Time</div>
                    <div className="text-2xl font-bold text-slate-200">
                      {metrics.average_recovery_time_minutes > 0
                        ? `${Math.round(metrics.average_recovery_time_minutes)} min`
                        : "N/A"}
                    </div>
                    <div className="text-xs text-slate-500 mt-1">
                      {metrics.recovery_times.length} recovery events
                    </div>
                  </div>
                  <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <div className="text-slate-400 text-sm mb-1">Total Open Events</div>
                    <div className="text-2xl font-bold text-red-400">{metrics.total_open_events}</div>
                    <div className="text-xs text-slate-500 mt-1">Last 7 days</div>
                  </div>
                  <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <div className="text-slate-400 text-sm mb-1">Total Close Events</div>
                    <div className="text-2xl font-bold text-emerald-400">{metrics.total_close_events}</div>
                    <div className="text-xs text-slate-500 mt-1">Last 7 days</div>
                  </div>
                </div>

                {/* Error Rate Trends */}
                {metrics.error_rate_trends.length > 0 && (
                  <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <h3 className="text-lg font-semibold text-slate-100 mb-4">Error Rate Trends</h3>
                    <div className="space-y-2">
                      {metrics.error_rate_trends.slice(-10).map((point, idx) => (
                        <div key={idx} className="flex items-center gap-4">
                          <div className="text-xs text-slate-400 w-32">
                            {format(new Date(point.timestamp), "MMM d, HH:mm")}
                          </div>
                          <div className="flex-1 bg-slate-800 rounded-full h-4 overflow-hidden">
                            <div
                              className={`h-full ${
                                point.error_rate > 10 ? "bg-red-500" : point.error_rate > 5 ? "bg-yellow-500" : "bg-emerald-500"
                              }`}
                              style={{ width: `${Math.min(point.error_rate, 100)}%` }}
                            />
                          </div>
                          <div className="text-sm text-slate-300 w-24 text-right">
                            {point.error_rate.toFixed(2)}%
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Recovery Times */}
                {metrics.recovery_times.length > 0 && (
                  <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                    <h3 className="text-lg font-semibold text-slate-100 mb-4 flex items-center gap-2">
                      <Clock className="text-emerald-400" size={18} />
                      Recovery Times
                    </h3>
                    <div className="space-y-2">
                      {metrics.recovery_times.slice(-10).map((recovery, idx) => (
                        <div key={idx} className="bg-slate-800 rounded-lg p-3 border border-slate-700">
                          <div className="flex items-center justify-between mb-2">
                            <div className="text-sm text-slate-300">
                              Opened: {format(new Date(recovery.opened_at), "MMM d, HH:mm")}
                            </div>
                            {recovery.recovery_time_minutes && (
                              <div className="text-sm font-semibold text-emerald-400">
                                Recovered in {Math.round(recovery.recovery_time_minutes)} min
                              </div>
                            )}
                          </div>
                          {recovery.closed_at && (
                            <div className="text-xs text-slate-400">
                              Closed: {format(new Date(recovery.closed_at), "MMM d, HH:mm")}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </>
            ) : (
              <div className="text-slate-400 text-center py-8">No metrics available for this policy</div>
            )}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
