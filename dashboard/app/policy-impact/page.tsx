"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { AlertTriangle, TrendingUp, Users, Globe, Activity, Play, RotateCcw, TestTube } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface PolicyImpactAnalytics {
  total_requests: number;
  requests_by_country: Record<string, number>;
  requests_by_agent: Record<string, {
    total: number;
    by_country: Record<string, number>;
  }>;
  requests_by_endpoint: Record<string, number>;
  risk_assessment: {
    critical_agents: string[];
    partial_impact: string[];
  };
}

interface SimulationResult {
  policy_type: string;
  total_requests: number;
  would_block: number;
  would_allow: number;
  affected_agents: Array<{
    agent_id: string;
    total_requests: number;
    would_block: number;
    would_allow: number;
    block_percentage: number;
    affected_endpoints: string[];
  }>;
  affected_endpoints: Record<string, number>;
  requests_by_country: Record<string, number>;
  estimated_impact: "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";
  critical_agents: string[];
  partial_impact_agents: string[];
  simulation_timestamp: string;
}

interface CostImpact {
  estimated_latency_cost_usd: number;
  estimated_throughput_cost_usd: number;
  estimated_total_cost_usd: number;
  average_latency_ms: number;
  estimated_blocked_rps: number;
  cost_per_ms_per_request: number;
  cost_per_rps: number;
  note: string;
}

interface BlastRadiusEntry {
  business_function: string;
  location: string;
  critical_count: number;
  partial_count: number;
  affected_agents: string[];
}

async function fetchPolicyImpact(timeRange: string = "30d") {
  const days = timeRange === "7d" ? 7 : timeRange === "30d" ? 30 : 90;
  const res = await fetch(`${API_BASE}/analytics/policy-impact?time_range=${days}`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch analytics: ${res.status}`);
  }
  return res.json() as Promise<PolicyImpactAnalytics>;
}

async function simulatePolicy(policyType: string, config: any, timeRangeDays: number = 7) {
  const res = await fetch(`${API_BASE}/policies/simulate`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      policy_type: policyType,
      policy_config: config,
      time_range_days: timeRangeDays,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    const error = await res.json();
    throw new Error(error.message || `Failed to simulate policy: ${res.status}`);
  }
  return res.json() as Promise<SimulationResult>;
}

async function rollbackPolicy(policyId: string, targetVersion?: number, notes?: string) {
  const res = await fetch(`${API_BASE}/policies/${policyId}/rollback`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      target_version: targetVersion,
      notes: notes,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    const error = await res.json();
    throw new Error(error.message || `Failed to rollback policy: ${res.status}`);
  }
  return res.json();
}

export default function PolicyImpactPage() {
  const [timeRange, setTimeRange] = useState("30d");
  const [simulationResult, setSimulationResult] = useState<SimulationResult | null>(null);
  const [costImpact, setCostImpact] = useState<CostImpact | null>(null);
  const [blastRadius, setBlastRadius] = useState<Record<string, BlastRadiusEntry>>({});
  const [isSimulating, setIsSimulating] = useState(false);
  const [simulationConfig, setSimulationConfig] = useState({
    blocked_countries: ["US", "CN", "RU"],
  });

  const queryClient = useQueryClient();

  const { data: analytics, isLoading, refetch } = useQuery({
    queryKey: ["policy-impact", timeRange],
    queryFn: () => fetchPolicyImpact(timeRange),
  });

  const rollbackMutation = useMutation({
    mutationFn: ({ policyId, targetVersion, notes }: { policyId: string; targetVersion?: number; notes?: string }) =>
      rollbackPolicy(policyId, targetVersion, notes),
    onSuccess: () => {
      alert("Policy rolled back successfully");
      queryClient.invalidateQueries({ queryKey: ["policy-impact"] });
    },
    onError: (error: Error) => {
      alert(`Rollback failed: ${error.message}`);
    },
  });

  const handleSimulate = async () => {
    setIsSimulating(true);
    try {
      const days = timeRange === "7d" ? 7 : timeRange === "30d" ? 30 : 90;
      const res = await fetch(`${API_BASE}/policies/preview-impact?policy_type=SOVEREIGN_LOCK&policy_config=${encodeURIComponent(JSON.stringify(simulationConfig))}&time_range_days=${days}`, {
        headers: getAuthHeaders(),
      });
      if (!res.ok) {
        throw new Error(`Failed to preview impact: ${res.status}`);
      }
      const data = await res.json();
      setSimulationResult(data.simulation_result);
      setCostImpact(data.cost_impact);
      setBlastRadius(data.blast_radius || {});
    } catch (error: any) {
      alert(`Simulation failed: ${error.message}`);
    } finally {
      setIsSimulating(false);
    }
  };

  const getImpactColor = (impact: string) => {
    switch (impact) {
      case "CRITICAL":
        return "text-red-400 bg-red-900/20 border-red-800";
      case "HIGH":
        return "text-orange-400 bg-orange-900/20 border-orange-800";
      case "MEDIUM":
        return "text-yellow-400 bg-yellow-900/20 border-yellow-800";
      case "LOW":
        return "text-green-400 bg-green-900/20 border-green-800";
      default:
        return "text-slate-400 bg-slate-900/20 border-slate-800";
    }
  };

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading analytics...</div>
        </div>
      </DashboardLayout>
    );
  }

  const blockedCountries = ["US", "CN", "RU"];
  const wouldBlockCount = analytics
    ? Object.entries(analytics.requests_by_country)
        .filter(([country]) => blockedCountries.includes(country))
        .reduce((sum, [, count]) => sum + count, 0)
    : 0;

  const wouldAllowCount = analytics ? (analytics.total_requests - wouldBlockCount) : 0;
  const blockPercentage = analytics && analytics.total_requests > 0
    ? ((wouldBlockCount / analytics.total_requests) * 100).toFixed(1)
    : "0";

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">
              Policy Impact Analysis
            </h1>
            <p className="text-slate-400">
              Operational Safety: Simulate policy changes before enforcement
            </p>
          </div>
          <div className="flex gap-2">
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value)}
              className="px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
            >
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
              <option value="90d">Last 90 days</option>
            </select>
          </div>
        </div>

        {/* Simulation Panel */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-white flex items-center gap-2">
              <TestTube size={20} />
              Policy Simulation
            </h2>
            <button
              onClick={handleSimulate}
              disabled={isSimulating}
              className="flex items-center gap-2 px-4 py-2 bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 rounded-lg transition-colors disabled:opacity-50"
            >
              <Play size={18} />
              {isSimulating ? "Simulating..." : "Run Simulation"}
            </button>
          </div>
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm text-slate-400 mb-2">Blocked Countries</label>
              <input
                type="text"
                value={simulationConfig.blocked_countries.join(", ")}
                onChange={(e) =>
                  setSimulationConfig({
                    ...simulationConfig,
                    blocked_countries: e.target.value.split(",").map((s) => s.trim().toUpperCase()),
                  })
                }
                placeholder="US, CN, RU"
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
              />
            </div>
          </div>
        </div>

        {/* Current Analytics Overview */}
        {analytics && (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-4">
              <div className="flex items-center gap-2 text-slate-400 mb-2">
                <Activity size={18} />
                <span className="text-sm">Total Requests</span>
              </div>
              <div className="text-2xl font-bold text-white">{analytics.total_requests.toLocaleString()}</div>
            </div>
            <div className="bg-red-900/20 border border-red-800 rounded-lg p-4">
              <div className="flex items-center gap-2 text-red-400 mb-2">
                <AlertTriangle size={18} />
                <span className="text-sm">Would Block</span>
              </div>
              <div className="text-2xl font-bold text-red-400">{wouldBlockCount.toLocaleString()}</div>
              <div className="text-xs text-red-500 mt-1">{blockPercentage}% of traffic</div>
            </div>
            <div className="bg-green-900/20 border border-green-800 rounded-lg p-4">
              <div className="flex items-center gap-2 text-green-400 mb-2">
                <TrendingUp size={18} />
                <span className="text-sm">Would Allow</span>
              </div>
              <div className="text-2xl font-bold text-green-400">{wouldAllowCount.toLocaleString()}</div>
            </div>
            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-4">
              <div className="flex items-center gap-2 text-slate-400 mb-2">
                <Users size={18} />
                <span className="text-sm">Affected Agents</span>
              </div>
              <div className="text-2xl font-bold text-white">
                {analytics.risk_assessment.critical_agents.length + analytics.risk_assessment.partial_impact.length}
              </div>
            </div>
          </div>
        )}

        {/* Simulation Results */}
        {simulationResult && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold text-white">Simulation Results</h2>
              <div className={`px-3 py-1 rounded-lg border ${getImpactColor(simulationResult.estimated_impact)}`}>
                {simulationResult.estimated_impact} Impact
              </div>
            </div>
            <div className="grid grid-cols-3 gap-4 mb-6">
              <div>
                <div className="text-sm text-slate-400 mb-1">Total Requests</div>
                <div className="text-2xl font-bold text-white">{simulationResult.total_requests.toLocaleString()}</div>
              </div>
              <div>
                <div className="text-sm text-red-400 mb-1">Would Block</div>
                <div className="text-2xl font-bold text-red-400">{simulationResult.would_block.toLocaleString()}</div>
              </div>
              <div>
                <div className="text-sm text-green-400 mb-1">Would Allow</div>
                <div className="text-2xl font-bold text-green-400">{simulationResult.would_allow.toLocaleString()}</div>
              </div>
            </div>

            {/* Critical Agents */}
            {simulationResult.critical_agents.length > 0 && (
              <div className="mb-6">
                <h3 className="text-lg font-semibold text-red-400 mb-3 flex items-center gap-2">
                  <AlertTriangle size={18} />
                  Critical Agents (100% Blocked)
                </h3>
                <div className="flex flex-wrap gap-2">
                  {simulationResult.critical_agents.map((agent) => (
                    <span
                      key={agent}
                      className="px-3 py-1 bg-red-900/30 border border-red-800 rounded-lg text-red-400 text-sm"
                    >
                      {agent}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Affected Agents Table */}
            <div className="mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">Agent Impact Analysis</h3>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-slate-700">
                      <th className="text-left py-2 px-4 text-slate-400">Agent ID</th>
                      <th className="text-right py-2 px-4 text-slate-400">Total</th>
                      <th className="text-right py-2 px-4 text-slate-400">Would Block</th>
                      <th className="text-right py-2 px-4 text-slate-400">Would Allow</th>
                      <th className="text-right py-2 px-4 text-slate-400">Block %</th>
                    </tr>
                  </thead>
                  <tbody>
                    {simulationResult.affected_agents.slice(0, 10).map((agent) => (
                      <tr key={agent.agent_id} className="border-b border-slate-800">
                        <td className="py-2 px-4 text-white">{agent.agent_id}</td>
                        <td className="py-2 px-4 text-right text-slate-300">{agent.total_requests}</td>
                        <td className="py-2 px-4 text-right text-red-400">{agent.would_block}</td>
                        <td className="py-2 px-4 text-right text-green-400">{agent.would_allow}</td>
                        <td className="py-2 px-4 text-right">
                          <span
                            className={
                              agent.block_percentage >= 50
                                ? "text-red-400"
                                : agent.block_percentage > 0
                                ? "text-yellow-400"
                                : "text-green-400"
                            }
                          >
                            {agent.block_percentage.toFixed(1)}%
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Requests by Country */}
            <div>
              <h3 className="text-lg font-semibold text-white mb-3">Requests by Country</h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                {Object.entries(simulationResult.requests_by_country)
                  .sort(([, a], [, b]) => b - a)
                  .slice(0, 8)
                  .map(([country, count]) => (
                    <div
                      key={country}
                      className={`p-3 rounded-lg border ${
                        blockedCountries.includes(country)
                          ? "bg-red-900/20 border-red-800 text-red-400"
                          : "bg-green-900/20 border-green-800 text-green-400"
                      }`}
                    >
                      <div className="text-sm mb-1">{country}</div>
                      <div className="text-xl font-bold">{count.toLocaleString()}</div>
                    </div>
                  ))}
              </div>
            </div>
          </div>
        )}

        {/* Requests by Country (Current) */}
        {analytics && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
              <Globe size={20} />
              Current Traffic by Country
            </h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {Object.entries(analytics.requests_by_country)
                .sort(([, a], [, b]) => b - a)
                .map(([country, count]) => {
                  const isBlocked = blockedCountries.includes(country);
                  const percentage = ((count / analytics.total_requests) * 100).toFixed(1);
                  return (
                    <div
                      key={country}
                      className={`p-4 rounded-lg border ${
                        isBlocked
                          ? "bg-red-900/20 border-red-800"
                          : "bg-green-900/20 border-green-800"
                      }`}
                    >
                      <div className={`text-sm mb-1 ${isBlocked ? "text-red-400" : "text-green-400"}`}>
                        {country}
                      </div>
                      <div className={`text-2xl font-bold ${isBlocked ? "text-red-400" : "text-green-400"}`}>
                        {count.toLocaleString()}
                      </div>
                      <div className="text-xs text-slate-500 mt-1">{percentage}%</div>
                    </div>
                  );
                })}
            </div>
          </div>
        )}

        {/* Affected Agents */}
        {analytics && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-white mb-4">Agent Traffic Analysis</h2>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-slate-700">
                    <th className="text-left py-2 px-4 text-slate-400">Agent ID</th>
                    <th className="text-right py-2 px-4 text-slate-400">Total Requests</th>
                    <th className="text-right py-2 px-4 text-slate-400">US</th>
                    <th className="text-right py-2 px-4 text-slate-400">CN</th>
                    <th className="text-right py-2 px-4 text-slate-400">RU</th>
                    <th className="text-right py-2 px-4 text-slate-400">EU/EEA</th>
                    <th className="text-right py-2 px-4 text-slate-400">Risk</th>
                  </tr>
                </thead>
                <tbody>
                  {Object.entries(analytics.requests_by_agent)
                    .sort(([, a], [, b]) => b.total - a.total)
                    .slice(0, 20)
                    .map(([agentId, stats]) => {
                      const blockedCount =
                        (stats.by_country["US"] || 0) +
                        (stats.by_country["CN"] || 0) +
                        (stats.by_country["RU"] || 0);
                      const blockPercentage = stats.total > 0 ? (blockedCount / stats.total) * 100 : 0;
                      const isCritical = blockPercentage >= 50;
                      const isPartial = blockPercentage > 0 && blockPercentage < 50;

                      return (
                        <tr key={agentId} className="border-b border-slate-800">
                          <td className="py-2 px-4 text-white">{agentId}</td>
                          <td className="py-2 px-4 text-right text-slate-300">{stats.total}</td>
                          <td className="py-2 px-4 text-right text-red-400">{stats.by_country["US"] || 0}</td>
                          <td className="py-2 px-4 text-right text-red-400">{stats.by_country["CN"] || 0}</td>
                          <td className="py-2 px-4 text-right text-red-400">{stats.by_country["RU"] || 0}</td>
                          <td className="py-2 px-4 text-right text-green-400">
                            {stats.total - blockedCount}
                          </td>
                          <td className="py-2 px-4 text-right">
                            {isCritical ? (
                              <span className="px-2 py-1 bg-red-900/30 border border-red-800 rounded text-red-400 text-xs">
                                CRITICAL
                              </span>
                            ) : isPartial ? (
                              <span className="px-2 py-1 bg-yellow-900/30 border border-yellow-800 rounded text-yellow-400 text-xs">
                                PARTIAL
                              </span>
                            ) : (
                              <span className="px-2 py-1 bg-green-900/30 border border-green-800 rounded text-green-400 text-xs">
                                SAFE
                              </span>
                            )}
                          </td>
                        </tr>
                      );
                    })}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

