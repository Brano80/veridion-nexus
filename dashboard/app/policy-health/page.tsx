"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Activity, AlertTriangle, CheckCircle, TrendingUp, Clock, Zap } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";
import { format } from "date-fns";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface PolicyHealthDashboard {
  policies: Array<{
    policy_id: string;
    policy_name: string;
    policy_type: string;
    health_status: string;
    success_rate: number;
    error_rate: number;
    total_requests: number;
    avg_latency_ms: number | null;
    circuit_breaker_state: string | null;
    last_updated: string;
  }>;
  total_policies: number;
  healthy_policies: number;
  degraded_policies: number;
  critical_policies: number;
  overall_health_score: number;
}

interface PolicyHealthTrends {
  policy_id: string;
  policy_name: string;
  trends: Array<{
    timestamp: string;
    success_rate: number;
    error_rate: number;
    total_requests: number;
    avg_latency_ms: number | null;
    health_status: string;
  }>;
}

async function fetchPolicyHealthDashboard(): Promise<PolicyHealthDashboard> {
  const res = await fetch(`${API_BASE}/analytics/policy-health`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch policy health: ${res.status}`);
  }
  return res.json();
}

async function fetchPolicyHealthTrends(policyId: string, timeRange: number = 24): Promise<PolicyHealthTrends> {
  const res = await fetch(`${API_BASE}/analytics/policy-health/${policyId}/trends?time_range=${timeRange}`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch trends: ${res.status}`);
  }
  return res.json();
}

export default function PolicyHealthPage() {
  const [selectedPolicyId, setSelectedPolicyId] = useState<string | null>(null);
  const [timeRange, setTimeRange] = useState<number>(24);

  const { data: dashboard, isLoading, error } = useQuery({
    queryKey: ["policy-health-dashboard"],
    queryFn: fetchPolicyHealthDashboard,
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const { data: trends, isLoading: loadingTrends } = useQuery({
    queryKey: ["policy-health-trends", selectedPolicyId, timeRange],
    queryFn: () => fetchPolicyHealthTrends(selectedPolicyId!, timeRange),
    enabled: !!selectedPolicyId,
    refetchInterval: 60000,
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

  if (!dashboard) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="text-slate-400">No policy health data available</div>
        </div>
      </DashboardLayout>
    );
  }

  const getHealthColor = (status: string) => {
    switch (status) {
      case "HEALTHY":
        return "bg-emerald-900/30 text-emerald-400 border-emerald-800";
      case "DEGRADED":
        return "bg-yellow-900/30 text-yellow-400 border-yellow-800";
      case "CRITICAL":
        return "bg-red-900/30 text-red-400 border-red-800";
      default:
        return "bg-slate-700 text-slate-400";
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
              Policy Health Dashboard
            </h1>
            <p className="text-slate-400 mt-2">
              Real-time monitoring of all active policies
            </p>
          </div>
          <div className="text-sm text-slate-400">
            Last updated: {format(new Date(), "HH:mm:ss")}
          </div>
        </div>

        {/* Overall Health Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Overall Health</span>
              <Activity className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">
              {dashboard.overall_health_score.toFixed(1)}%
            </div>
            <div className="text-xs text-slate-500 mt-1">Average success rate</div>
          </div>

          <div className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-emerald-400 text-sm">Healthy</span>
              <CheckCircle className="text-emerald-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-emerald-400">{dashboard.healthy_policies}</div>
            <div className="text-xs text-emerald-500 mt-1">Policies operating normally</div>
          </div>

          <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-yellow-400 text-sm">Degraded</span>
              <AlertTriangle className="text-yellow-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-yellow-400">{dashboard.degraded_policies}</div>
            <div className="text-xs text-yellow-500 mt-1">Require attention</div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">Critical</span>
              <AlertTriangle className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{dashboard.critical_policies}</div>
            <div className="text-xs text-red-500 mt-1">Immediate action required</div>
          </div>
        </div>

        {/* Policy List */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Activity className="text-emerald-400" size={20} />
            Policy Health Status
          </h2>
          {dashboard.policies.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No active policies</div>
          ) : (
            <div className="space-y-3">
              {dashboard.policies.map((policy) => (
                <div
                  key={policy.policy_id}
                  className={`bg-slate-900/50 rounded-lg p-4 border cursor-pointer transition-colors ${
                    selectedPolicyId === policy.policy_id
                      ? "border-emerald-500"
                      : "border-slate-700"
                  }`}
                  onClick={() => setSelectedPolicyId(
                    selectedPolicyId === policy.policy_id ? null : policy.policy_id
                  )}
                >
                  <div className="flex items-center justify-between mb-3">
                    <div>
                      <span className="font-semibold text-slate-200">{policy.policy_name}</span>
                      <span className="text-xs text-slate-400 ml-2 font-mono">
                        {policy.policy_type}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${getHealthColor(policy.health_status)}`}>
                        {policy.health_status}
                      </span>
                      {policy.circuit_breaker_state === "OPEN" && (
                        <span className="px-3 py-1 rounded-full text-xs font-semibold bg-red-900/30 text-red-400 border border-red-800">
                          CIRCUIT OPEN
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-3">
                    <div>
                      <span className="text-slate-400">Success Rate:</span>
                      <div className={`font-semibold ${
                        policy.success_rate >= 95 ? "text-emerald-400" :
                        policy.success_rate >= 90 ? "text-yellow-400" : "text-red-400"
                      }`}>
                        {policy.success_rate.toFixed(1)}%
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Error Rate:</span>
                      <div className={`font-semibold ${
                        policy.error_rate < 5 ? "text-emerald-400" :
                        policy.error_rate < 10 ? "text-yellow-400" : "text-red-400"
                      }`}>
                        {policy.error_rate.toFixed(1)}%
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Total Requests:</span>
                      <div className="font-semibold text-slate-200">
                        {policy.total_requests.toLocaleString()}
                      </div>
                    </div>
                    {policy.avg_latency_ms && (
                      <div>
                        <span className="text-slate-400">Avg Latency:</span>
                        <div className="font-semibold text-slate-200 flex items-center gap-1">
                          <Clock size={14} />
                          {policy.avg_latency_ms.toFixed(0)}ms
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Health Progress Bar */}
                  <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div
                      className={`h-full ${
                        policy.success_rate >= 95 ? "bg-emerald-500" :
                        policy.success_rate >= 90 ? "bg-yellow-500" : "bg-red-500"
                      }`}
                      style={{ width: `${policy.success_rate}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Trends View */}
        {selectedPolicyId && trends && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold text-slate-100 flex items-center gap-2">
                <TrendingUp className="text-emerald-400" size={20} />
                Health Trends: {trends.policy_name}
              </h2>
              <select
                value={timeRange}
                onChange={(e) => setTimeRange(Number(e.target.value))}
                className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200"
              >
                <option value={6}>Last 6 hours</option>
                <option value={24}>Last 24 hours</option>
                <option value={48}>Last 48 hours</option>
                <option value={168}>Last 7 days</option>
              </select>
            </div>
            {loadingTrends ? (
              <div className="text-slate-400 text-center py-8">Loading trends...</div>
            ) : trends.trends.length === 0 ? (
              <div className="text-slate-400 text-center py-8">No trend data available</div>
            ) : (
              <div className="space-y-2">
                {trends.trends.map((trend, idx) => (
                  <div key={idx} className="bg-slate-900/50 rounded-lg p-3 border border-slate-700">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm text-slate-300">
                        {format(new Date(trend.timestamp), "MMM d, HH:mm")}
                      </span>
                      <span className={`px-2 py-1 rounded text-xs font-semibold ${getHealthColor(trend.health_status)}`}>
                        {trend.health_status}
                      </span>
                    </div>
                    <div className="grid grid-cols-4 gap-2 text-xs">
                      <div>
                        <span className="text-slate-400">Success:</span>
                        <div className="text-emerald-400 font-semibold">{trend.success_rate.toFixed(1)}%</div>
                      </div>
                      <div>
                        <span className="text-slate-400">Error:</span>
                        <div className="text-red-400 font-semibold">{trend.error_rate.toFixed(1)}%</div>
                      </div>
                      <div>
                        <span className="text-slate-400">Requests:</span>
                        <div className="text-slate-200 font-semibold">{trend.total_requests}</div>
                      </div>
                      {trend.avg_latency_ms && (
                        <div>
                          <span className="text-slate-400">Latency:</span>
                          <div className="text-slate-200 font-semibold flex items-center gap-1">
                            <Zap size={12} />
                            {trend.avg_latency_ms.toFixed(0)}ms
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

