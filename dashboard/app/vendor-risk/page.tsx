"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { format } from "date-fns";
import { AlertTriangle, Globe, Building2, Shield, TrendingUp, CheckCircle, XCircle, BarChart3 } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface TPRMComplianceReport {
  total_vendors: number;
  high_risk_vendors: number;
  critical_risk_vendors: number;
  non_compliant_vendors: number;
  vendors_by_country: Record<string, number>;
  vendors_by_risk_level: Record<string, number>;
  vendors: Array<{
    vendor_domain: string;
    vendor_name: string | null;
    risk_score: number;
    risk_level: string;
    compliance_status: string;
    country_code: string | null;
    industry_sector: string | null;
    associated_assets: string[];
    last_assessed: string | null;
  }>;
  compliance_score: number;
  dora_article9_compliant: boolean;
}

async function fetchVendorRiskDashboard(riskLevel?: string, country?: string): Promise<TPRMComplianceReport> {
  const params = new URLSearchParams();
  if (riskLevel) params.append("risk_level", riskLevel);
  if (country) params.append("country", country);
  
  const res = await fetch(`${API_BASE}/analytics/vendor-risk?${params}`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch vendor risk data: ${res.status}`);
  }
  return res.json();
}

export default function VendorRiskPage() {
  const [riskFilter, setRiskFilter] = useState<string>("");
  const [countryFilter, setCountryFilter] = useState<string>("");

  const { data: dashboard, isLoading, error } = useQuery({
    queryKey: ["vendor-risk-dashboard", riskFilter, countryFilter],
    queryFn: () => fetchVendorRiskDashboard(riskFilter || undefined, countryFilter || undefined),
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
          <div className="text-slate-400">No vendor risk data available</div>
        </div>
      </DashboardLayout>
    );
  }

  const getRiskColor = (riskLevel: string) => {
    switch (riskLevel) {
      case "CRITICAL":
        return "bg-red-900/30 text-red-400 border-red-800";
      case "HIGH":
        return "bg-orange-900/30 text-orange-400 border-orange-800";
      case "MEDIUM":
        return "bg-yellow-900/30 text-yellow-400 border-yellow-800";
      case "LOW":
        return "bg-emerald-900/30 text-emerald-400 border-emerald-800";
      default:
        return "bg-slate-700 text-slate-400";
    }
  };

  const getComplianceColor = (status: string) => {
    switch (status) {
      case "COMPLIANT":
        return "text-emerald-400";
      case "NON_COMPLIANT":
        return "text-red-400";
      default:
        return "text-yellow-400";
    }
  };

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
              <Building2 className="text-emerald-400" size={32} />
              Vendor Risk Dashboard
            </h1>
            <p className="text-slate-400 mt-2">
              Third-Party Risk Management (TPRM) - DORA Article 9 Compliance
            </p>
          </div>
          <div className="flex gap-2">
            <select
              value={riskFilter}
              onChange={(e) => setRiskFilter(e.target.value)}
              className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200"
            >
              <option value="">All Risk Levels</option>
              <option value="CRITICAL">Critical</option>
              <option value="HIGH">High</option>
              <option value="MEDIUM">Medium</option>
              <option value="LOW">Low</option>
            </select>
            <input
              type="text"
              placeholder="Filter by country..."
              value={countryFilter}
              onChange={(e) => setCountryFilter(e.target.value)}
              className="bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-slate-200 w-48"
            />
          </div>
        </div>

        {/* DORA Compliance Banner */}
        <div className={`rounded-lg p-4 border ${
          dashboard.dora_article9_compliant
            ? "bg-emerald-900/20 border-emerald-800 text-emerald-400"
            : "bg-red-900/20 border-red-800 text-red-400"
        }`}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Shield size={20} />
              <span className="font-semibold">
                DORA Article 9 Compliance: {dashboard.dora_article9_compliant ? "COMPLIANT" : "NON-COMPLIANT"}
              </span>
            </div>
            <span className="text-sm">
              {dashboard.dora_article9_compliant
                ? "Third-party risk register is complete and compliant"
                : "Risk register needs attention - compliance score below 80%"}
            </span>
          </div>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Total Vendors</span>
              <Building2 className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{dashboard.total_vendors}</div>
            <div className="text-xs text-slate-500 mt-1">In risk register</div>
          </div>

          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-red-400 text-sm">Critical Risk</span>
              <AlertTriangle className="text-red-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-red-400">{dashboard.critical_risk_vendors}</div>
            <div className="text-xs text-red-500 mt-1">Require immediate attention</div>
          </div>

          <div className="bg-orange-900/20 border border-orange-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-orange-400 text-sm">High Risk</span>
              <TrendingUp className="text-orange-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-orange-400">{dashboard.high_risk_vendors}</div>
            <div className="text-xs text-orange-500 mt-1">Monitor closely</div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Compliance Score</span>
              <BarChart3 className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{dashboard.compliance_score.toFixed(1)}%</div>
            <div className="text-xs text-slate-500 mt-1">
              {dashboard.compliance_score >= 80 ? "✓ Compliant" : "⚠ Needs improvement"}
            </div>
          </div>
        </div>

        {/* Vendors by Risk Level */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <BarChart3 className="text-emerald-400" size={20} />
            Vendors by Risk Level
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {Object.entries(dashboard.vendors_by_risk_level).map(([level, count]) => (
              <div key={level} className={`rounded-lg p-4 border ${getRiskColor(level)}`}>
                <div className="text-2xl font-bold">{count}</div>
                <div className="text-sm mt-1">{level}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Vendors by Country */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Globe className="text-emerald-400" size={20} />
            Vendors by Country
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {Object.entries(dashboard.vendors_by_country)
              .sort(([, a], [, b]) => b - a)
              .slice(0, 12)
              .map(([country, count]) => (
                <div key={country} className="bg-slate-900/50 rounded-lg p-3 border border-slate-700">
                  <div className="text-lg font-bold text-slate-200">{country}</div>
                  <div className="text-sm text-slate-400">{count} vendor{count !== 1 ? "s" : ""}</div>
                </div>
              ))}
          </div>
        </div>

        {/* Vendor List */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <Building2 className="text-emerald-400" size={20} />
            Vendor Risk Details
          </h2>
          {dashboard.vendors.length === 0 ? (
            <div className="text-slate-400 text-center py-8">No vendors found</div>
          ) : (
            <div className="space-y-3">
              {dashboard.vendors.map((vendor) => (
                <div key={vendor.vendor_domain} className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-3">
                    <div>
                      <span className="font-semibold text-slate-200">{vendor.vendor_name || vendor.vendor_domain}</span>
                      <span className="text-xs text-slate-400 ml-2 font-mono">{vendor.vendor_domain}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${getRiskColor(vendor.risk_level)}`}>
                        {vendor.risk_level}
                      </span>
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold ${getComplianceColor(vendor.compliance_status)}`}>
                        {vendor.compliance_status}
                      </span>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-3">
                    <div>
                      <span className="text-slate-400">Risk Score:</span>
                      <div className={`font-semibold ${
                        vendor.risk_score >= 80 ? "text-red-400" :
                        vendor.risk_score >= 60 ? "text-orange-400" :
                        vendor.risk_score >= 40 ? "text-yellow-400" : "text-emerald-400"
                      }`}>
                        {vendor.risk_score.toFixed(1)}/100
                      </div>
                    </div>
                    <div>
                      <span className="text-slate-400">Country:</span>
                      <div className="font-semibold text-slate-200">{vendor.country_code || "UNKNOWN"}</div>
                    </div>
                    <div>
                      <span className="text-slate-400">Industry:</span>
                      <div className="font-semibold text-slate-200">{vendor.industry_sector || "UNKNOWN"}</div>
                    </div>
                    {vendor.last_assessed && (
                      <div>
                        <span className="text-slate-400">Last Assessed:</span>
                        <div className="font-semibold text-slate-200 text-xs">
                          {format(new Date(vendor.last_assessed), "MMM d, yyyy")}
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="h-2 bg-slate-700 rounded-full overflow-hidden mb-3">
                    <div
                      className={`h-full ${
                        vendor.risk_score >= 80 ? "bg-red-500" :
                        vendor.risk_score >= 60 ? "bg-orange-500" :
                        vendor.risk_score >= 40 ? "bg-yellow-500" : "bg-emerald-500"
                      }`}
                      style={{ width: `${vendor.risk_score}%` }}
                    />
                  </div>

                  {vendor.associated_assets.length > 0 && (
                    <div className="text-xs text-slate-400">
                      <span className="font-semibold">Associated Assets:</span> {vendor.associated_assets.join(", ")}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

