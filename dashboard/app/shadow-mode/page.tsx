"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { format } from "date-fns";
import { Eye, AlertTriangle, TrendingUp, Users, Globe, Activity, Shield, CheckCircle, XCircle, BarChart3, Download } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface ShadowModeAnalytics {
  total_logs: number;
  would_block_count: number;
  would_allow_count: number;
  block_percentage: number;
  top_blocked_agents: Array<{
    agent_id: string;
    would_block: number;
    would_allow: number;
    total: number;
    block_percentage: number;
  }>;
  top_blocked_regions: Array<{
    region: string;
    would_block: number;
    would_allow: number;
    total: number;
    block_percentage: number;
  }>;
  top_policies_applied: Array<{
    policy_name: string;
    would_block: number;
    would_allow: number;
    total: number;
    block_percentage: number;
  }>;
  time_range: {
    start: string;
    end: string;
    days: number;
  };
  confidence_score: number;
}

async function fetchShadowModeAnalytics(days: number = 7, agentId?: string) {
  const params = new URLSearchParams({ days: days.toString() });
  if (agentId) {
    params.append("agent_id", agentId);
  }
  const res = await fetch(`${API_BASE}/analytics/shadow-mode?${params}`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch shadow mode analytics: ${res.status}`);
  }
  return res.json() as Promise<ShadowModeAnalytics>;
}

export default function ShadowModePage() {
  const [timeRange, setTimeRange] = useState<number>(7);
  const [agentFilter, setAgentFilter] = useState<string>("");

  const { data: analytics, isLoading, error, refetch } = useQuery({
    queryKey: ["shadow-mode-analytics", timeRange, agentFilter],
    queryFn: () => fetchShadowModeAnalytics(timeRange, agentFilter || undefined),
    refetchInterval: 30000, // Refresh every 30 seconds
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
          <div className="text-slate-400">No shadow mode data available</div>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
              <Eye className="text-emerald-400" size={32} />
              Shadow Mode Analytics
            </h1>
            <p className="text-slate-400 mt-2">
              Preview what would be blocked in enforcing mode without affecting production
            </p>
          </div>
          <div className="flex gap-2">
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(Number(e.target.value))}
              className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200"
            >
              <option value={7}>Last 7 days</option>
              <option value={30}>Last 30 days</option>
              <option value={90}>Last 90 days</option>
            </select>
            <input
              type="text"
              placeholder="Filter by agent ID..."
              value={agentFilter}
              onChange={(e) => setAgentFilter(e.target.value)}
              className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200 w-64"
            />
            <button
              onClick={() => {
                const params = new URLSearchParams({ 
                  format: "csv", 
                  days: timeRange.toString() 
                });
                if (agentFilter) {
                  params.append("agent_id", agentFilter);
                }
                window.open(`${API_BASE}/analytics/shadow-mode/export?${params}`, "_blank");
              }}
              className="flex items-center gap-2 px-4 py-2 bg-emerald-900/30 hover:bg-emerald-800/50 border border-emerald-800 rounded-lg text-emerald-400 transition-colors"
            >
              <Download size={18} />
              Export CSV
            </button>
            <button
              onClick={() => {
                const params = new URLSearchParams({ 
                  format: "json", 
                  days: timeRange.toString() 
                });
                if (agentFilter) {
                  params.append("agent_id", agentFilter);
                }
                window.open(`${API_BASE}/analytics/shadow-mode/export?${params}`, "_blank");
              }}
              className="flex items-center gap-2 px-4 py-2 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg text-slate-300 transition-colors"
            >
              <Download size={18} />
              Export JSON
            </button>
            <button
              onClick={() => {
                const params = new URLSearchParams({ 
                  format: "pdf", 
                  days: timeRange.toString() 
                });
                if (agentFilter) {
                  params.append("agent_id", agentFilter);
                }
                window.open(`${API_BASE}/analytics/shadow-mode/export?${params}`, "_blank");
              }}
              className="flex items-center gap-2 px-4 py-2 bg-red-900/30 hover:bg-red-800/50 border border-red-800 rounded-lg text-red-400 transition-colors"
            >
              <Download size={18} />
              Export PDF
            </button>
          </div>
        </div>

        {/* Confidence Score Banner */}
        <div className={`rounded-lg p-4 border ${
          analytics.confidence_score >= 90
            ? "bg-emerald-900/20 border-emerald-800 text-emerald-400"
            : analytics.confidence_score >= 70
            ? "bg-yellow-900/20 border-yellow-800 text-yellow-400"
            : "bg-orange-900/20 border-orange-800 text-orange-400"
        }`}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <BarChart3 size={20} />
              <span className="font-semibold">Confidence Score: {analytics.confidence_score.toFixed(1)}%</span>
            </div>
            <span className="text-sm">
              {analytics.confidence_score >= 90
                ? "High confidence - Ready for enforcement"
                : analytics.confidence_score >= 70
                ? "Medium confidence - Consider more data"
                : "Low confidence - Need more shadow mode data"}
            </span>
          </div>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Total Logs</span>
              <Activity className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{analytics.total_logs.toLocaleString()}</div>
            <div className="text-xs text-slate-500 mt-1">
              {format(new Date(analytics.time_range.start), "MMM d")} - {format(new Date(analytics.time_range.end), "MMM d")}
            </div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">Would Block</span>
              <XCircle className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{analytics.would_block_count.toLocaleString()}</div>
            <div className="text-xs text-red-500 mt-1">
              {analytics.block_percentage.toFixed(1)}% of total
            </div>
          </div>

          <div className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-emerald-400 text-sm">Would Allow</span>
              <CheckCircle className="text-emerald-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-emerald-400">{analytics.would_allow_count.toLocaleString()}</div>
            <div className="text-xs text-emerald-500 mt-1">
              {(100 - analytics.block_percentage).toFixed(1)}% of total
            </div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Block Rate</span>
              <TrendingUp className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{analytics.block_percentage.toFixed(1)}%</div>
            <div className="text-xs text-slate-500 mt-1">
              {analytics.block_percentage > 20 ? "⚠️ High block rate" : analytics.block_percentage > 5 ? "⚠️ Medium block rate" : "✓ Low block rate"}
            </div>
          </div>
        </div>

        {/* Top Blocked Agents */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Users className="text-emerald-400" size={20} />
            Top Blocked Agents
          </h2>
          {analytics.top_blocked_agents.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No blocked agents in this time period</div>
          ) : (
            <div className="space-y-3">
              {analytics.top_blocked_agents.map((agent) => (
                <div key={agent.agent_id} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-mono text-sm text-slate-300">{agent.agent_id}</span>
                    <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                      agent.block_percentage >= 50
                        ? "bg-red-900/30 text-red-400 border border-red-800"
                        : agent.block_percentage >= 20
                        ? "bg-yellow-900/30 text-yellow-400 border border-yellow-800"
                        : "bg-slate-700 text-slate-400"
                    }`}>
                      {agent.block_percentage.toFixed(1)}% blocked
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-red-400">
                      <XCircle className="inline mr-1" size={14} />
                      {agent.would_block} blocked
                    </span>
                    <span className="text-emerald-400">
                      <CheckCircle className="inline mr-1" size={14} />
                      {agent.would_allow} allowed
                    </span>
                    <span className="text-slate-400">
                      Total: {agent.total}
                    </span>
                  </div>
                  <div className="mt-2 h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-red-500"
                      style={{ width: `${agent.block_percentage}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Top Blocked Regions */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Globe className="text-emerald-400" size={20} />
            Top Blocked Regions
          </h2>
          {analytics.top_blocked_regions.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No blocked regions in this time period</div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {analytics.top_blocked_regions.map((region) => (
                <div key={region.region} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-semibold text-slate-200">{region.region}</span>
                    <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                      region.block_percentage >= 50
                        ? "bg-red-900/30 text-red-400 border border-red-800"
                        : "bg-slate-700 text-slate-400"
                    }`}>
                      {region.block_percentage.toFixed(1)}%
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-red-400">{region.would_block} blocked</span>
                    <span className="text-emerald-400">{region.would_allow} allowed</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Top Policies Applied */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Shield className="text-emerald-400" size={20} />
            Policies Applied
          </h2>
          {analytics.top_policies_applied.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No policies applied in this time period</div>
          ) : (
            <div className="space-y-3">
              {analytics.top_policies_applied.map((policy) => (
                <div key={policy.policy_name} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-semibold text-slate-200">{policy.policy_name}</span>
                    <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                      policy.block_percentage >= 50
                        ? "bg-red-900/30 text-red-400 border border-red-800"
                        : "bg-slate-700 text-slate-400"
                    }`}>
                      {policy.block_percentage.toFixed(1)}% blocked
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-red-400">{policy.would_block} blocked</span>
                    <span className="text-emerald-400">{policy.would_allow} allowed</span>
                    <span className="text-slate-400">Total: {policy.total}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Warning Banner */}
        {analytics.block_percentage > 20 && (
          <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="text-yellow-400 mt-0.5" size={20} />
              <div>
                <h3 className="font-semibold text-yellow-400 mb-1">High Block Rate Detected</h3>
                <p className="text-yellow-300 text-sm">
                  {analytics.block_percentage.toFixed(1)}% of requests would be blocked in enforcing mode. 
                  Review the policies and consider adjusting them before switching to ENFORCING mode to avoid production disruption.
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

