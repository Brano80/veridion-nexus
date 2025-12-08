"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { AlertTriangle, TrendingUp, BarChart3 } from "lucide-react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchRiskData() {
  const res = await fetch(`${API_BASE}/risks`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch risks: ${res.status}`);
  }
  const data = await res.json();
  return data.data || [];
}

export default function RiskAssessmentPage() {
  const { data: risks, isLoading } = useQuery({
    queryKey: ["risk-assessments"],
    queryFn: fetchRiskData,
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

  // Process data for charts
  const riskDistribution = (risks || []).reduce(
    (acc: any, risk: any) => {
      acc[risk.risk_level] = (acc[risk.risk_level] || 0) + 1;
      return acc;
    },
    { HIGH: 0, MEDIUM: 0, LOW: 0 }
  );

  const chartData = [
    { name: "High", value: riskDistribution?.HIGH || 0, color: "#ef4444" },
    { name: "Medium", value: riskDistribution?.MEDIUM || 0, color: "#f97316" },
    { name: "Low", value: riskDistribution?.LOW || 0, color: "#10b981" },
  ];

  const timelineData = (risks || []).slice(0, 10).map((risk: any) => ({
    date: risk.assessed_at?.split(" ")[0] || "N/A",
    high: risk.risk_level === "HIGH" ? 1 : 0,
    medium: risk.risk_level === "MEDIUM" ? 1 : 0,
    low: risk.risk_level === "LOW" ? 1 : 0,
  }));

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">
            Risk Assessment Dashboard
          </h1>
          <p className="text-slate-400">
            EU AI Act Article 9 - Risk assessment and monitoring
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-red-900/20 border border-red-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <AlertTriangle className="text-red-400" size={24} />
              <div className="text-3xl font-bold text-red-400">
                {riskDistribution?.HIGH || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">High Risk Items</div>
          </div>
          <div className="p-6 bg-orange-900/20 border border-orange-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <TrendingUp className="text-orange-400" size={24} />
              <div className="text-3xl font-bold text-orange-400">
                {riskDistribution?.MEDIUM || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Medium Risk Items</div>
          </div>
          <div className="p-6 bg-emerald-900/20 border border-emerald-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <BarChart3 className="text-emerald-400" size={24} />
              <div className="text-3xl font-bold text-emerald-400">
                {riskDistribution?.LOW || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Low Risk Items</div>
          </div>
        </div>

        {/* Charts */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Risk Distribution Pie Chart */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <h2 className="text-xl font-bold text-white mb-4">
              Risk Distribution
            </h2>
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={chartData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={({ name, percent }) =>
                    `${name} ${(percent * 100).toFixed(0)}%`
                  }
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {chartData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          </div>

          {/* Risk Timeline */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <h2 className="text-xl font-bold text-white mb-4">
              Risk Timeline
            </h2>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={timelineData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="date" stroke="#9ca3af" />
                <YAxis stroke="#9ca3af" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1e293b",
                    border: "1px solid #334155",
                    color: "#e2e8f0",
                  }}
                />
                <Legend />
                <Bar dataKey="high" stackId="a" fill="#ef4444" />
                <Bar dataKey="medium" stackId="a" fill="#f97316" />
                <Bar dataKey="low" stackId="a" fill="#10b981" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Risk List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">All Risk Assessments</h2>
          <div className="space-y-3">
            {risks?.map((risk: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          risk.risk_level === "HIGH"
                            ? "bg-red-900/30 text-red-400"
                            : risk.risk_level === "MEDIUM"
                            ? "bg-orange-900/30 text-orange-400"
                            : "bg-emerald-900/30 text-emerald-400"
                        }`}
                      >
                        {risk.risk_level}
                      </span>
                      <span className="text-xs text-slate-500">
                        {risk.assessed_at}
                      </span>
                    </div>
                    {risk.risk_factors?.length > 0 && (
                      <div className="text-sm text-slate-300 mt-2">
                        <span className="text-slate-500">Risk Factors: </span>
                        {risk.risk_factors.join(", ")}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

