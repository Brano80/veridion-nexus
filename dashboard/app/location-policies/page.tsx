"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Globe, AlertTriangle, Shield, MapPin, TrendingUp } from "lucide-react";
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

async function fetchLocationAnalytics(): Promise<PolicyImpactAnalytics> {
  const res = await fetch(`${API_BASE}/analytics/policy-impact?time_range=30`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch location analytics: ${res.status}`);
  }
  return res.json();
}

const EU_COUNTRIES = ["AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", 
  "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL", 
  "PL", "PT", "RO", "SK", "SI", "ES", "SE"];

const HIGH_RISK_COUNTRIES = ["US", "CN", "RU", "IN"];

export default function LocationPoliciesPage() {
  const { data: analytics, isLoading, error } = useQuery({
    queryKey: ["location-analytics"],
    queryFn: fetchLocationAnalytics,
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

  if (!analytics) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="text-slate-400">No location analytics data available</div>
        </div>
      </DashboardLayout>
    );
  }

  // Calculate location risk scores
  const locationStats = Object.entries(analytics.requests_by_country)
    .map(([country, count]) => {
      const percentage = (count / analytics.total_requests) * 100;
      const isEU = EU_COUNTRIES.includes(country);
      const isHighRisk = HIGH_RISK_COUNTRIES.includes(country);
      
      let riskScore = 0;
      if (isHighRisk) riskScore = 80;
      else if (!isEU) riskScore = 60;
      else riskScore = 20;

      return {
        country,
        count,
        percentage,
        isEU,
        isHighRisk,
        riskScore,
      };
    })
    .sort((a, b) => b.count - a.count);

  const euTraffic = locationStats
    .filter(s => s.isEU)
    .reduce((sum, s) => sum + s.count, 0);
  const euPercentage = (euTraffic / analytics.total_requests) * 100;

  const highRiskTraffic = locationStats
    .filter(s => s.isHighRisk)
    .reduce((sum, s) => sum + s.count, 0);
  const highRiskPercentage = (highRiskTraffic / analytics.total_requests) * 100;

  const getRiskColor = (riskScore: number) => {
    if (riskScore >= 70) return "bg-red-900/30 text-red-400 border-red-800";
    if (riskScore >= 50) return "bg-orange-900/30 text-orange-400 border-orange-800";
    if (riskScore >= 30) return "bg-yellow-900/30 text-yellow-400 border-yellow-800";
    return "bg-emerald-900/30 text-emerald-400 border-emerald-800";
  };

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
            <Globe className="text-emerald-400" size={32} />
            Location-Based Policy Recommendations
          </h1>
          <p className="text-slate-400 mt-2">
            Geographic risk analysis and policy recommendations based on traffic patterns
          </p>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">EU Traffic</span>
              <Shield className="text-emerald-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-emerald-400">{euPercentage.toFixed(1)}%</div>
            <div className="text-xs text-slate-500 mt-1">{euTraffic.toLocaleString()} requests</div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">High-Risk Traffic</span>
              <AlertTriangle className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{highRiskPercentage.toFixed(1)}%</div>
            <div className="text-xs text-red-500 mt-1">{highRiskTraffic.toLocaleString()} requests</div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Total Requests</span>
              <TrendingUp className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{analytics.total_requests.toLocaleString()}</div>
            <div className="text-xs text-slate-500 mt-1">Last 30 days</div>
          </div>
        </div>

        {/* Recommendations */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <MapPin className="text-emerald-400" size={20} />
            Policy Recommendations
          </h2>
          <div className="space-y-3">
            {highRiskPercentage > 10 && (
              <div className="bg-red-900/20 border border-red-800 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <AlertTriangle className="text-red-400 mt-1" size={20} />
                  <div>
                    <div className="font-semibold text-red-400 mb-1">
                      Block High-Risk Countries
                    </div>
                    <div className="text-sm text-slate-300">
                      {highRiskPercentage.toFixed(1)}% of traffic originates from high-risk countries (US, CN, RU, IN).
                      Consider implementing a SOVEREIGN_LOCK policy to block these regions.
                    </div>
                  </div>
                </div>
              </div>
            )}

            {euPercentage < 80 && (
              <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <Shield className="text-yellow-400 mt-1" size={20} />
                  <div>
                    <div className="font-semibold text-yellow-400 mb-1">
                      Increase EU Traffic
                    </div>
                    <div className="text-sm text-slate-300">
                      Only {euPercentage.toFixed(1)}% of traffic is EU-based. Consider redirecting non-EU traffic
                      to EU regions to improve compliance.
                    </div>
                  </div>
                </div>
              </div>
            )}

            {analytics.risk_assessment.critical_agents.length > 0 && (
              <div className="bg-orange-900/20 border border-orange-800 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <TrendingUp className="text-orange-400 mt-1" size={20} />
                  <div>
                    <div className="font-semibold text-orange-400 mb-1">
                      Critical Agents Detected
                    </div>
                    <div className="text-sm text-slate-300">
                      {analytics.risk_assessment.critical_agents.length} agent(s) have &gt;50% traffic from blocked countries.
                      Review these agents and consider location-based policies.
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Location Breakdown */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Globe className="text-emerald-400" size={20} />
            Traffic by Country
          </h2>
          <div className="space-y-3">
            {locationStats.map((stat) => (
              <div key={stat.country} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-slate-200">{stat.country}</span>
                    {stat.isEU && (
                      <span className="px-2 py-1 rounded text-xs bg-emerald-900/30 text-emerald-400 border border-emerald-800">
                        EU
                      </span>
                    )}
                    {stat.isHighRisk && (
                      <span className="px-2 py-1 rounded text-xs bg-red-900/30 text-red-400 border border-red-800">
                        HIGH RISK
                      </span>
                    )}
                  </div>
                  <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${getRiskColor(stat.riskScore)}`}>
                    Risk: {stat.riskScore}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-4 text-sm mb-2">
                  <div>
                    <span className="text-slate-400">Requests:</span>
                    <div className="font-semibold text-slate-200">{stat.count.toLocaleString()}</div>
                  </div>
                  <div>
                    <span className="text-slate-400">Percentage:</span>
                    <div className="font-semibold text-slate-200">{stat.percentage.toFixed(2)}%</div>
                  </div>
                </div>
                <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                  <div
                    className={`h-full ${
                      stat.riskScore >= 70 ? "bg-red-500" :
                      stat.riskScore >= 50 ? "bg-orange-500" :
                      stat.riskScore >= 30 ? "bg-yellow-500" : "bg-emerald-500"
                    }`}
                    style={{ width: `${stat.percentage}%` }}
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

