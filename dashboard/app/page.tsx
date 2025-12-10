"use client";

import DashboardLayout from "./components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Activity,
  Shield,
  AlertTriangle,
  CheckCircle,
  Clock,
  TrendingUp,
  Eye,
  EyeOff,
  AlertCircle,
  Zap,
} from "lucide-react";
import { getAuthHeaders } from "./utils/auth";
import { useState } from "react";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchCanaryHealth() {
  const headers = getAuthHeaders();
  const res = await fetch(`${API_BASE}/analytics/canary`, { headers });
  if (!res.ok) {
    return null;
  }
  return res.json();
}

interface EnforcementMode {
  enforcement_mode: string;
  enabled_at: string;
  enabled_by: string | null;
  description: string | null;
}

async function fetchEnforcementMode(): Promise<EnforcementMode> {
  const res = await fetch(`${API_BASE}/system/enforcement-mode`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch enforcement mode: ${res.status}`);
  }
  return res.json();
}

async function setEnforcementMode(mode: string, description?: string): Promise<EnforcementMode> {
  const res = await fetch(`${API_BASE}/system/enforcement-mode`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      enforcement_mode: mode,
      description: description || `Changed to ${mode} mode`,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    const error = await res.json();
    throw new Error(error.message || `Failed to set enforcement mode: ${res.status}`);
  }
  return res.json();
}

async function fetchStats() {
  const headers = getAuthHeaders();
  
  const [logsRes, risksRes, breachesRes, monitoring] = await Promise.all([
    fetch(`${API_BASE}/logs`, { headers }).then((r) => r.json()),
    fetch(`${API_BASE}/risks`, { headers }).then((r) => r.json()).catch(() => ({ data: [] })),
    fetch(`${API_BASE}/breaches`, { headers }).then((r) => r.json()).catch(() => ({ data: [] })),
    fetch(`${API_BASE}/monitoring/events`, { headers }).then((r) => r.json()).catch(() => ({ events: [] })),
  ]);

  // API vracia { data: [...], pagination: {...} }
  const logs = logsRes.data || [];
  const risks = risksRes.data || risksRes || [];
  const breaches = breachesRes.data || breachesRes || [];

  const highRisks = risks.filter((r: any) => r.risk_level === "HIGH").length;
  const openBreaches = breaches.filter((b: any) => b.status === "REPORTED").length;
  const openEvents = monitoring.events?.filter(
    (e: any) => e.resolution_status === "OPEN"
  ).length || 0;

  return {
    totalRecords: logs.length,
    highRisks,
    openBreaches,
    openEvents,
    recentActivity: logs.slice(0, 5),
  };
}

export default function Home() {
  const queryClient = useQueryClient();
  const [showModeConfirm, setShowModeConfirm] = useState(false);
  const [pendingMode, setPendingMode] = useState<string | null>(null);

  const { data: stats, isLoading } = useQuery({
    queryKey: ["dashboard-stats"],
    queryFn: fetchStats,
  });

  const { data: enforcementMode, isLoading: modeLoading } = useQuery({
    queryKey: ["enforcement-mode"],
    queryFn: fetchEnforcementMode,
    refetchInterval: 10000, // Refresh every 10 seconds
  });

  const { data: canaryHealth } = useQuery({
    queryKey: ["canary-health"],
    queryFn: fetchCanaryHealth,
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const setModeMutation = useMutation({
    mutationFn: ({ mode, description }: { mode: string; description?: string }) =>
      setEnforcementMode(mode, description),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["enforcement-mode"] });
      queryClient.invalidateQueries({ queryKey: ["dashboard-stats"] });
      setShowModeConfirm(false);
      setPendingMode(null);
    },
    onError: (error: Error) => {
      alert(`Failed to update enforcement mode: ${error.message}`);
      setShowModeConfirm(false);
      setPendingMode(null);
    },
  });

  const handleModeChange = (newMode: string) => {
    if (newMode === "ENFORCING") {
      setPendingMode(newMode);
      setShowModeConfirm(true);
    } else {
      setModeMutation.mutate({ 
        mode: newMode, 
        description: `Switched to ${newMode} mode from dashboard` 
      });
    }
  };

  const confirmEnforce = () => {
    if (pendingMode) {
      setModeMutation.mutate({ 
        mode: pendingMode, 
        description: "Switched to ENFORCING mode from dashboard - Shadow mode testing complete" 
      });
    }
  };

  if (isLoading || modeLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading...</div>
        </div>
      </DashboardLayout>
    );
  }

  const statCards = [
    {
      title: "Total Records",
      value: stats?.totalRecords || 0,
      icon: Activity,
      color: "emerald",
    },
    {
      title: "High Risk Items",
      value: stats?.highRisks || 0,
      icon: AlertTriangle,
      color: "red",
    },
    {
      title: "Open Breaches",
      value: stats?.openBreaches || 0,
      icon: Shield,
      color: "orange",
    },
    {
      title: "Monitoring Events",
      value: stats?.openEvents || 0,
      icon: Clock,
      color: "blue",
    },
  ];

  const isShadowMode = enforcementMode?.enforcement_mode === "SHADOW" || enforcementMode?.enforcement_mode === "DRY_RUN";
  const isEnforcing = enforcementMode?.enforcement_mode === "ENFORCING";

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Shadow Mode Banner */}
        {isShadowMode && (
          <div className="bg-yellow-900/30 border-2 border-yellow-800 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <EyeOff className="text-yellow-400" size={24} />
                <div>
                  <div className="flex items-center gap-2">
                    <span className="text-lg font-bold text-yellow-400">
                      SHADOW MODE ACTIVE
                    </span>
                    <span className="px-2 py-1 bg-yellow-900/50 border border-yellow-800 rounded text-xs font-medium text-yellow-300">
                      TESTING
                    </span>
                  </div>
                  <p className="text-sm text-yellow-300/80 mt-1">
                    Policies are being logged but not enforced. No requests are being blocked.
                  </p>
                </div>
              </div>
              <button
                onClick={() => handleModeChange("ENFORCING")}
                disabled={setModeMutation.isPending}
                className="flex items-center gap-2 px-4 py-2 bg-red-900/50 hover:bg-red-800/70 text-red-300 border border-red-800 rounded-lg transition-colors disabled:opacity-50 font-medium"
              >
                <Zap size={18} />
                Switch to ENFORCING
              </button>
            </div>
          </div>
        )}

        {/* Enforcing Mode Indicator */}
        {isEnforcing && (
          <div className="bg-red-900/20 border border-red-800 rounded-lg p-3">
            <div className="flex items-center gap-2">
              <Shield className="text-red-400" size={20} />
              <span className="text-sm font-medium text-red-400">
                ENFORCING MODE: All policies are active and blocking non-compliant requests
              </span>
            </div>
          </div>
        )}

        {/* Mode Change Confirmation Dialog */}
        {showModeConfirm && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-900 border border-red-800 rounded-lg p-6 max-w-md w-full mx-4">
              <div className="flex items-center gap-3 mb-4">
                <AlertCircle className="text-red-400" size={24} />
                <h3 className="text-xl font-bold text-white">Switch to ENFORCING Mode?</h3>
              </div>
              <p className="text-slate-300 mb-6">
                This will activate all policies and start blocking non-compliant requests. 
                Make sure you've reviewed shadow mode analytics and are ready for production enforcement.
              </p>
              <div className="flex gap-3">
                <button
                  onClick={confirmEnforce}
                  disabled={setModeMutation.isPending}
                  className="flex-1 px-4 py-2 bg-red-900/50 hover:bg-red-800/70 text-red-300 border border-red-800 rounded-lg transition-colors disabled:opacity-50 font-medium"
                >
                  {setModeMutation.isPending ? "Switching..." : "Yes, Switch to ENFORCING"}
                </button>
                <button
                  onClick={() => {
                    setShowModeConfirm(false);
                    setPendingMode(null);
                  }}
                  disabled={setModeMutation.isPending}
                  className="flex-1 px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors disabled:opacity-50"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">Dashboard</h1>
            <p className="text-slate-400">
              Overview of compliance and monitoring activities
            </p>
          </div>
          {/* Quick Mode Toggle */}
          {enforcementMode && (
            <div className="flex items-center gap-2">
              <div className={`px-3 py-1.5 rounded-lg border text-sm font-medium ${
                isShadowMode 
                  ? "bg-yellow-900/30 border-yellow-800 text-yellow-400"
                  : isEnforcing
                  ? "bg-red-900/30 border-red-800 text-red-400"
                  : "bg-blue-900/30 border-blue-800 text-blue-400"
              }`}>
                {enforcementMode.enforcement_mode}
              </div>
              {isShadowMode && (
                <button
                  onClick={() => handleModeChange("ENFORCING")}
                  disabled={setModeMutation.isPending}
                  className="flex items-center gap-2 px-3 py-1.5 bg-red-900/50 hover:bg-red-800/70 text-red-300 border border-red-800 rounded-lg transition-colors disabled:opacity-50 text-sm font-medium"
                >
                  <Zap size={16} />
                  Enforce
                </button>
              )}
            </div>
          )}
        </div>

        {/* Canary Health Widget */}
        {canaryHealth && canaryHealth.active_canaries > 0 && (
          <div className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <BarChart3 className="text-emerald-400" size={20} />
                <div>
                  <div className="text-sm font-medium text-emerald-400">Active Canary Deployments</div>
                  <div className="text-xs text-emerald-300/80">
                    {canaryHealth.active_canaries} canary{canaryHealth.active_canaries !== 1 ? 'ies' : ''} running
                    {canaryHealth.recent_promotions?.length > 0 && ` • ${canaryHealth.recent_promotions.length} promotion${canaryHealth.recent_promotions.length !== 1 ? 's' : ''} in last 24h`}
                  </div>
                </div>
              </div>
              <a
                href="/canary"
                className="px-3 py-1.5 bg-emerald-900/50 hover:bg-emerald-800/70 text-emerald-300 border border-emerald-800 rounded-lg transition-colors text-sm font-medium"
              >
                View Details
              </a>
            </div>
          </div>
        )}

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {statCards.map((stat) => {
            const Icon = stat.icon;
            const colorClasses = {
              emerald: "bg-emerald-900/20 border-emerald-800 text-emerald-400",
              red: "bg-red-900/20 border-red-800 text-red-400",
              orange: "bg-orange-900/20 border-orange-800 text-orange-400",
              blue: "bg-blue-900/20 border-blue-800 text-blue-400",
            };
            return (
              <div
                key={stat.title}
                className={`p-6 rounded-lg border ${colorClasses[stat.color as keyof typeof colorClasses]}`}
              >
                <div className="flex items-center justify-between mb-4">
                  <Icon size={24} />
                  <TrendingUp size={16} className="opacity-50" />
                </div>
                <div className="text-3xl font-bold mb-1">{stat.value}</div>
                <div className="text-sm opacity-75">{stat.title}</div>
              </div>
            );
          })}
        </div>

        {/* Recent Activity */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Recent Activity</h2>
          {stats?.recentActivity && stats.recentActivity.length > 0 ? (
            <div className="space-y-3">
              {stats.recentActivity.map((log: any, i: number) => (
                <div
                  key={i}
                  className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg border border-slate-700"
                >
                  <div className="flex items-center gap-3">
                    <CheckCircle
                      size={16}
                      className={
                        log.status?.includes("BLOCKED")
                          ? "text-red-400"
                          : "text-emerald-400"
                      }
                    />
                    <div>
                      <div className="text-sm font-medium text-white">
                        {log.action_summary || log.agent_id || "Unknown action"}
                      </div>
                      <div className="text-xs text-slate-500">
                        {log.timestamp || log.created_at || "No timestamp"}
                        {log.target_region && ` • Region: ${log.target_region}`}
                      </div>
                    </div>
                  </div>
                  <span
                    className={`px-2 py-1 rounded text-xs font-medium ${
                      log.status?.includes("BLOCKED")
                        ? "bg-red-900/30 text-red-400"
                        : "bg-emerald-900/30 text-emerald-400"
                    }`}
                  >
                    {log.status || "UNKNOWN"}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-slate-400 text-center py-8">
              No recent activity
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}
