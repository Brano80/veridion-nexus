"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { format } from "date-fns";
import { TrendingUp, TrendingDown, Activity, ArrowUp, ArrowDown, CheckCircle, XCircle, BarChart3 } from "lucide-react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface CanaryAnalytics {
  total_policies: number;
  active_canaries: number;
  policies: Array<{
    policy_id: string;
    policy_type: string;
    current_traffic_percentage: number;
    total_requests: number;
    successful_requests: number;
    failed_requests: number;
    blocked_requests: number;
    success_rate: number;
    auto_promote_enabled: boolean;
    auto_rollback_enabled: boolean;
    promotion_threshold: number;
    rollback_threshold: number;
    min_requests_for_promotion: number;
    evaluation_window_minutes: number;
    last_evaluated_at: string | null;
  }>;
  recent_promotions: Array<{
    policy_id: string;
    policy_type: string;
    from_percentage: number;
    to_percentage: number;
    reason: string;
    success_rate: number;
    total_requests: number;
    timestamp: string;
  }>;
  recent_rollbacks: Array<{
    policy_id: string;
    policy_type: string;
    from_percentage: number;
    to_percentage: number;
    reason: string;
    success_rate: number;
    total_requests: number;
    timestamp: string;
  }>;
}

async function fetchCanaryAnalytics(): Promise<CanaryAnalytics> {
  const res = await fetch(`${API_BASE}/analytics/canary`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch canary analytics: ${res.status}`);
  }
  return res.json();
}

async function updateCanaryTraffic(policyId: string, trafficPercentage: number) {
  const res = await fetch(`${API_BASE}/policies/${policyId}/canary-config`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      traffic_percentage: trafficPercentage,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    const error = await res.json();
    throw new Error(error.message || `Failed to update canary traffic: ${res.status}`);
  }
  return res.json();
}

export default function CanaryPage() {
  const queryClient = useQueryClient();
  const { data: analytics, isLoading, error } = useQuery({
    queryKey: ["canary-analytics"],
    queryFn: fetchCanaryAnalytics,
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const updateTrafficMutation = useMutation({
    mutationFn: ({ policyId, percentage }: { policyId: string; percentage: number }) =>
      updateCanaryTraffic(policyId, percentage),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["canary-analytics"] });
      alert("Canary traffic percentage updated successfully");
    },
    onError: (error: Error) => {
      alert(`Failed to update canary traffic: ${error.message}`);
    },
  });

  const rolloutTemplates = [1, 5, 10, 25, 50, 100];

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
          <div className="text-slate-400">No canary deployment data available</div>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
            <BarChart3 className="text-emerald-400" size={32} />
            Canary Deployment Dashboard
          </h1>
          <p className="text-slate-400 mt-2">
            Monitor gradual policy rollouts with automatic promotion and rollback
          </p>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Active Canaries</span>
              <Activity className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{analytics.active_canaries}</div>
            <div className="text-xs text-slate-500 mt-1">Out of {analytics.total_policies} total policies</div>
          </div>

          <div className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-emerald-400 text-sm">Recent Promotions</span>
              <TrendingUp className="text-emerald-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-emerald-400">{analytics.recent_promotions.length}</div>
            <div className="text-xs text-emerald-500 mt-1">Last 24 hours</div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">Recent Rollbacks</span>
              <TrendingDown className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{analytics.recent_rollbacks.length}</div>
            <div className="text-xs text-red-500 mt-1">Last 24 hours</div>
          </div>
        </div>

        {/* Active Canaries */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Activity className="text-emerald-400" size={20} />
            Active Canary Deployments
          </h2>
          {analytics.policies.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No active canary deployments</div>
          ) : (
            <div className="space-y-4">
              {analytics.policies.map((policy) => (
                <div key={policy.policy_id} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-3">
                    <div>
                      <span className="font-semibold text-slate-200">{policy.policy_type}</span>
                      <span className="text-xs text-slate-400 ml-2 font-mono">{policy.policy_id.slice(0, 8)}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-2xl font-bold text-emerald-400">{policy.current_traffic_percentage}%</span>
                      <span className="text-sm text-slate-400">traffic</span>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-3">
                    <div>
                      <span className="text-slate-400">Success Rate:</span>
                      <div className={`font-semibold ${
                        policy.success_rate >= policy.promotion_threshold
                          ? "text-emerald-400"
                          : policy.success_rate < policy.rollback_threshold
                          ? "text-red-400"
                          : "text-yellow-400"
                      }`}>
                        {policy.success_rate.toFixed(2)}%
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Requests:</span>
                      <div className="font-semibold text-slate-200">
                        {policy.total_requests.toLocaleString()}
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Successful:</span>
                      <div className="font-semibold text-emerald-400">
                        {policy.successful_requests.toLocaleString()}
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Blocked:</span>
                      <div className="font-semibold text-red-400">
                        {policy.blocked_requests.toLocaleString()}
                      </div>
                    </div>
                  </div>

                  <div className="mb-3">
                    <div className="flex items-center justify-between text-xs text-slate-400 mb-1">
                      <span>Traffic Distribution</span>
                      <span>{policy.current_traffic_percentage}% canary / {100 - policy.current_traffic_percentage}% baseline</span>
                    </div>
                    <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-emerald-500"
                        style={{ width: `${policy.current_traffic_percentage}%` }}
                      />
                    </div>
                  </div>

                  <div className="mb-3">
                    <div className="flex items-center justify-between text-xs text-slate-400 mb-1">
                      <span>Success Rate vs Thresholds</span>
                      <span>Promote: {policy.promotion_threshold}% | Rollback: {policy.rollback_threshold}%</span>
                    </div>
                    <div className="h-3 bg-slate-700 rounded-full overflow-hidden relative">
                      <div
                        className="h-full bg-emerald-500"
                        style={{ width: `${policy.success_rate}%` }}
                      />
                      <div
                        className="absolute h-full w-0.5 bg-yellow-400"
                        style={{ left: `${policy.promotion_threshold}%` }}
                      />
                      <div
                        className="absolute h-full w-0.5 bg-red-400"
                        style={{ left: `${policy.rollback_threshold}%` }}
                      />
                    </div>
                  </div>

                  <div className="flex items-center gap-4 text-xs text-slate-400">
                    {policy.auto_promote_enabled && (
                      <span className="flex items-center gap-1">
                        <ArrowUp size={12} className="text-emerald-400" />
                        Auto-promote enabled
                      </span>
                    )}
                    {policy.auto_rollback_enabled && (
                      <span className="flex items-center gap-1">
                        <ArrowDown size={12} className="text-red-400" />
                        Auto-rollback enabled
                      </span>
                    )}
                    <span>Min requests: {policy.min_requests_for_promotion}</span>
                    <span>Window: {policy.evaluation_window_minutes} min</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Recent Promotions */}
        {analytics.recent_promotions.length > 0 && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <TrendingUp className="text-emerald-400" size={20} />
              Recent Promotions
            </h2>
            <div className="space-y-2">
              {analytics.recent_promotions.slice(0, 10).map((promo, idx) => (
                <div key={idx} className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <ArrowUp className="text-emerald-400" size={16} />
                      <div>
                        <span className="font-semibold text-slate-200">{promo.policy_type}</span>
                        <span className="text-xs text-slate-400 ml-2">
                          {promo.from_percentage}% → {promo.to_percentage}%
                        </span>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm text-emerald-400 font-semibold">
                        {promo.success_rate.toFixed(2)}% success
                      </div>
                      <div className="text-xs text-slate-400">
                        {format(new Date(promo.timestamp), "MMM d, HH:mm:ss")}
                      </div>
                    </div>
                  </div>
                  <div className="text-xs text-slate-400 mt-1">{promo.reason}</div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Recent Rollbacks */}
        {analytics.recent_rollbacks.length > 0 && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <TrendingDown className="text-red-400" size={20} />
              Recent Rollbacks
            </h2>
            <div className="space-y-2">
              {analytics.recent_rollbacks.slice(0, 10).map((rollback, idx) => (
                <div key={idx} className="bg-red-900/20 border border-red-800 rounded-lg p-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <ArrowDown className="text-red-400" size={16} />
                      <div>
                        <span className="font-semibold text-slate-200">{rollback.policy_type}</span>
                        <span className="text-xs text-slate-400 ml-2">
                          {rollback.from_percentage}% → {rollback.to_percentage}%
                        </span>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm text-red-400 font-semibold">
                        {rollback.success_rate.toFixed(2)}% success
                      </div>
                      <div className="text-xs text-slate-400">
                        {format(new Date(rollback.timestamp), "MMM d, HH:mm:ss")}
                      </div>
                    </div>
                  </div>
                  <div className="text-xs text-slate-400 mt-1">{rollback.reason}</div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

