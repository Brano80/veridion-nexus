"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { BarChart3, Building2, Shield, AlertTriangle, TrendingUp, CheckCircle, XCircle, PieChart } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface BusinessFunctionDashboard {
  business_functions: Array<{
    business_function: string;
    asset_count: number;
    compliant_count: number;
    non_compliant_count: number;
    compliance_score: number;
    high_risk_assets: number;
    critical_assets: number;
    avg_risk_score: number;
  }>;
  total_assets: number;
  compliant_assets: number;
  non_compliant_assets: number;
  compliance_by_function: Record<string, number>;
}

async function fetchBusinessFunctionDashboard(businessFunction?: string): Promise<BusinessFunctionDashboard> {
  const params = new URLSearchParams();
  if (businessFunction) params.append("business_function", businessFunction);
  
  const res = await fetch(`${API_BASE}/analytics/business-functions?${params}`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch business function data: ${res.status}`);
  }
  return res.json();
}

export default function BusinessFunctionsPage() {
  const [selectedFunction, setSelectedFunction] = useState<string>("");

  const { data: dashboard, isLoading, error } = useQuery({
    queryKey: ["business-function-dashboard", selectedFunction],
    queryFn: () => fetchBusinessFunctionDashboard(selectedFunction || undefined),
    refetchInterval: 60000, // Refresh every minute
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
          <div className="text-slate-400">No business function data available</div>
        </div>
      </DashboardLayout>
    );
  }

  const overallCompliance = dashboard.total_assets > 0
    ? (dashboard.compliant_assets / dashboard.total_assets) * 100
    : 100;

  const getComplianceColor = (score: number) => {
    if (score >= 90) return "text-emerald-400";
    if (score >= 70) return "text-yellow-400";
    return "text-red-400";
  };

  const getRiskColor = (riskScore: number) => {
    if (riskScore >= 80) return "bg-red-900/30 text-red-400 border-red-800";
    if (riskScore >= 60) return "bg-orange-900/30 text-orange-400 border-orange-800";
    if (riskScore >= 40) return "bg-yellow-900/30 text-yellow-400 border-yellow-800";
    return "bg-emerald-900/30 text-emerald-400 border-emerald-800";
  };

  const formatFunctionName = (name: string) => {
    return name
      .split("_")
      .map(word => word.charAt(0) + word.slice(1).toLowerCase())
      .join(" ");
  };

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
              <Building2 className="text-emerald-400" size={32} />
              Business Function Dashboard
            </h1>
            <p className="text-slate-400 mt-2">
              Compliance and risk metrics by business function
            </p>
          </div>
          <select
            value={selectedFunction}
            onChange={(e) => setSelectedFunction(e.target.value)}
            className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200"
          >
            <option value="">All Business Functions</option>
            {dashboard.business_functions.map((bf) => (
              <option key={bf.business_function} value={bf.business_function}>
                {formatFunctionName(bf.business_function)}
              </option>
            ))}
          </select>
        </div>

        {/* Overall Compliance Banner */}
        <div className={`rounded-lg p-4 border ${
          overallCompliance >= 90
            ? "bg-emerald-900/20 border-emerald-800 text-emerald-400"
            : overallCompliance >= 70
            ? "bg-yellow-900/20 border-yellow-800 text-yellow-400"
            : "bg-red-900/20 border-red-800 text-red-400"
        }`}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Shield size={20} />
              <span className="font-semibold">
                Overall Compliance: {overallCompliance.toFixed(1)}%
              </span>
            </div>
            <div className="flex items-center gap-4 text-sm">
              <span>
                <CheckCircle className="inline mr-1" size={16} />
                {dashboard.compliant_assets} Compliant
              </span>
              <span>
                <XCircle className="inline mr-1" size={16} />
                {dashboard.non_compliant_assets} Non-Compliant
              </span>
            </div>
          </div>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Total Assets</span>
              <Building2 className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{dashboard.total_assets}</div>
            <div className="text-xs text-slate-500 mt-1">Across all functions</div>
          </div>

          <div className="bg-emerald-900/20 border border-emerald-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-emerald-400 text-sm">Compliant Assets</span>
              <CheckCircle className="text-emerald-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-emerald-400">{dashboard.compliant_assets}</div>
            <div className="text-xs text-emerald-500 mt-1">
              {overallCompliance.toFixed(1)}% compliance rate
            </div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">Non-Compliant</span>
              <XCircle className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{dashboard.non_compliant_assets}</div>
            <div className="text-xs text-red-500 mt-1">Require attention</div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Business Functions</span>
              <PieChart className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{dashboard.business_functions.length}</div>
            <div className="text-xs text-slate-500 mt-1">Tracked functions</div>
          </div>
        </div>

        {/* Business Function List */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <BarChart3 className="text-emerald-400" size={20} />
            Compliance by Business Function
          </h2>
          {dashboard.business_functions.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No business function data available</div>
          ) : (
            <div className="space-y-3">
              {dashboard.business_functions.map((bf) => (
                <div key={bf.business_function} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-3">
                    <div>
                      <span className="font-semibold text-slate-200 text-lg">
                        {formatFunctionName(bf.business_function)}
                      </span>
                      <span className="text-xs text-slate-400 ml-2 font-mono">
                        {bf.business_function}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold ${getComplianceColor(bf.compliance_score)}`}>
                        {bf.compliance_score.toFixed(1)}% Compliant
                      </span>
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${getRiskColor(bf.avg_risk_score)}`}>
                        Risk: {bf.avg_risk_score.toFixed(1)}
                      </span>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-3">
                    <div>
                      <span className="text-slate-400">Total Assets:</span>
                      <div className="font-semibold text-slate-200">{bf.asset_count}</div>
                    </div>
                    <div>
                      <span className="text-slate-400">Compliant:</span>
                      <div className="font-semibold text-emerald-400">{bf.compliant_count}</div>
                    </div>
                    <div>
                      <span className="text-slate-400">High Risk:</span>
                      <div className="font-semibold text-orange-400">{bf.high_risk_assets}</div>
                    </div>
                    <div>
                      <span className="text-slate-400">Critical:</span>
                      <div className="font-semibold text-red-400">{bf.critical_assets}</div>
                    </div>
                  </div>

                  {/* Compliance Progress Bar */}
                  <div className="h-3 bg-slate-700 rounded-full overflow-hidden mb-2">
                    <div
                      className={`h-full ${
                        bf.compliance_score >= 90 ? "bg-emerald-500" :
                        bf.compliance_score >= 70 ? "bg-yellow-500" : "bg-red-500"
                      }`}
                      style={{ width: `${bf.compliance_score}%` }}
                    />
                  </div>

                  {/* Risk Score Bar */}
                  <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div
                      className={`h-full ${
                        bf.avg_risk_score >= 80 ? "bg-red-500" :
                        bf.avg_risk_score >= 60 ? "bg-orange-500" :
                        bf.avg_risk_score >= 40 ? "bg-yellow-500" : "bg-emerald-500"
                      }`}
                      style={{ width: `${bf.avg_risk_score}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Compliance Distribution Chart */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <PieChart className="text-emerald-400" size={20} />
            Compliance Distribution
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Object.entries(dashboard.compliance_by_function)
              .sort(([, a], [, b]) => b - a)
              .map(([functionName, score]) => (
                <div key={functionName} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-slate-300 font-medium">{formatFunctionName(functionName)}</span>
                    <span className={`font-semibold ${getComplianceColor(score)}`}>
                      {score.toFixed(1)}%
                    </span>
                  </div>
                  <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                    <div
                      className={`h-full ${
                        score >= 90 ? "bg-emerald-500" :
                        score >= 70 ? "bg-yellow-500" : "bg-red-500"
                      }`}
                      style={{ width: `${score}%` }}
                    />
                  </div>
                </div>
              ))}
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

