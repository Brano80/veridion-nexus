"use client";

import DashboardLayout from "./components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import {
  Activity,
  Shield,
  AlertTriangle,
  CheckCircle,
  Clock,
  TrendingUp,
} from "lucide-react";
import { getAuthHeaders } from "./utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

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
  const { data: stats, isLoading } = useQuery({
    queryKey: ["dashboard-stats"],
    queryFn: fetchStats,
  });

  if (isLoading) {
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

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Dashboard</h1>
          <p className="text-slate-400">
            Overview of compliance and monitoring activities
          </p>
        </div>

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
