"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Shield, TrendingUp, AlertTriangle, CheckCircle, XCircle, FileText, BarChart3, Target } from "lucide-react";
import { format } from "date-fns";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface ExecutiveScorecard {
  report_date: string;
  compliance_score: number;
  risk_level: string;
  liability_protection_status: string;
  nis2_readiness: number;
  dora_compliance: boolean;
  total_assets: number;
  compliant_assets: number;
  non_compliant_assets: number;
  critical_issues_count: number;
  high_risk_issues_count: number;
  last_incident_date: string | null;
  days_since_last_incident: number | null;
  executive_summary: string;
  recommendations: string[];
}

interface DORAReport {
  overall_score: number;
  article9_compliant: boolean;
  article10_compliant: boolean;
  article11_compliant: boolean;
  article9_score: number;
  article10_score: number;
  article11_score: number;
}

interface NIS2Report {
  overall_score: number;
  article20_compliant: boolean;
  article21_compliant: boolean;
  article23_compliant: boolean;
  article20_score: number;
  article21_score: number;
  article23_score: number;
  liability_protection_status: string;
}

async function fetchExecutiveScorecard(): Promise<ExecutiveScorecard> {
  const res = await fetch(`${API_BASE}/reports/executive-assurance`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

async function fetchDORAReport(): Promise<DORAReport> {
  const res = await fetch(`${API_BASE}/reports/dora-compliance`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

async function fetchNIS2Report(): Promise<NIS2Report> {
  const res = await fetch(`${API_BASE}/reports/nis2-compliance`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

export default function ExecutivePage() {
  const { data: scorecard, isLoading: loadingScorecard } = useQuery({
    queryKey: ["executive-scorecard"],
    queryFn: fetchExecutiveScorecard,
    refetchInterval: 300000, // 5 minutes
  });

  const { data: doraReport, isLoading: loadingDORA } = useQuery({
    queryKey: ["dora-report"],
    queryFn: fetchDORAReport,
    refetchInterval: 300000,
  });

  const { data: nis2Report, isLoading: loadingNIS2 } = useQuery({
    queryKey: ["nis2-report"],
    queryFn: fetchNIS2Report,
    refetchInterval: 300000,
  });

  const isLoading = loadingScorecard || loadingDORA || loadingNIS2;

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

  if (!scorecard || !doraReport || !nis2Report) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="text-slate-400">No executive data available</div>
        </div>
      </DashboardLayout>
    );
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case "PROTECTED":
        return "bg-emerald-900/30 text-emerald-400 border-emerald-800";
      case "AT_RISK":
        return "bg-yellow-900/30 text-yellow-400 border-yellow-800";
      case "EXPOSED":
        return "bg-red-900/30 text-red-400 border-red-800";
      default:
        return "bg-slate-700 text-slate-400";
    }
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case "LOW":
        return "text-emerald-400";
      case "MEDIUM":
        return "text-yellow-400";
      case "HIGH":
        return "text-orange-400";
      case "CRITICAL":
        return "text-red-400";
      default:
        return "text-slate-400";
    }
  };

  const overallCompliance = (scorecard.compliance_score + doraReport.overall_score + nis2Report.overall_score) / 3;

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-slate-100 flex items-center gap-3">
            <Shield className="text-emerald-400" size={32} />
            Executive Compliance Dashboard
          </h1>
          <p className="text-slate-400 mt-2">
            Board-level compliance overview • {format(new Date(scorecard.report_date), "MMMM d, yyyy")}
          </p>
        </div>

        {/* Liability Protection Banner */}
        <div className={`rounded-lg p-6 border ${getStatusColor(scorecard.liability_protection_status)}`}>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-semibold mb-1">Management Liability Protection Status</div>
              <div className="text-2xl font-bold">{scorecard.liability_protection_status}</div>
              <div className="text-sm mt-2 opacity-90">
                {scorecard.liability_protection_status === "PROTECTED" 
                  ? "Management is protected from personal liability under NIS2 Article 20"
                  : scorecard.liability_protection_status === "AT_RISK"
                  ? "Management liability protection is at risk - immediate action required"
                  : "Management is exposed to personal liability - critical action required"}
              </div>
            </div>
            <Shield size={48} className="opacity-50" />
          </div>
        </div>

        {/* Key Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Overall Compliance</span>
              <Target className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{overallCompliance.toFixed(1)}%</div>
            <div className="text-xs text-slate-500 mt-1">Combined score</div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Risk Level</span>
              <AlertTriangle className={`${getRiskColor(scorecard.risk_level)}`} size={20} />
            </div>
            <div className={`text-3xl font-bold ${getRiskColor(scorecard.risk_level)}`}>
              {scorecard.risk_level}
            </div>
            <div className="text-xs text-slate-500 mt-1">Current assessment</div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">NIS2 Readiness</span>
              <Shield className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">{scorecard.nis2_readiness.toFixed(1)}%</div>
            <div className="text-xs text-slate-500 mt-1">Article 20 compliance</div>
          </div>

          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-400 text-sm">Days Since Incident</span>
              <TrendingUp className="text-slate-500" size={20} />
            </div>
            <div className="text-3xl font-bold text-slate-100">
              {scorecard.days_since_last_incident ?? "N/A"}
            </div>
            <div className="text-xs text-slate-500 mt-1">Last incident tracking</div>
          </div>
        </div>

        {/* Compliance Scores */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* GDPR/EU AI Act Compliance */}
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <FileText className="text-emerald-400" size={20} />
              GDPR & EU AI Act
            </h3>
            <div className="text-4xl font-bold text-slate-100 mb-2">
              {scorecard.compliance_score.toFixed(1)}%
            </div>
            <div className="h-3 bg-slate-700 rounded-full overflow-hidden mb-4">
              <div
                className={`h-full ${
                  scorecard.compliance_score >= 90 ? "bg-emerald-500" :
                  scorecard.compliance_score >= 70 ? "bg-yellow-500" : "bg-red-500"
                }`}
                style={{ width: `${scorecard.compliance_score}%` }}
              />
            </div>
            <div className="text-sm text-slate-400">
              {scorecard.compliant_assets} of {scorecard.total_assets} assets compliant
            </div>
          </div>

          {/* DORA Compliance */}
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <Shield className="text-emerald-400" size={20} />
              DORA Compliance
            </h3>
            <div className="text-4xl font-bold text-slate-100 mb-2">
              {doraReport.overall_score.toFixed(1)}%
            </div>
            <div className="h-3 bg-slate-700 rounded-full overflow-hidden mb-4">
              <div
                className={`h-full ${
                  doraReport.overall_score >= 90 ? "bg-emerald-500" :
                  doraReport.overall_score >= 70 ? "bg-yellow-500" : "bg-red-500"
                }`}
                style={{ width: `${doraReport.overall_score}%` }}
              />
            </div>
            <div className="space-y-1 text-xs">
              <div className="flex items-center gap-2">
                {doraReport.article9_compliant ? (
                  <CheckCircle className="text-emerald-400" size={14} />
                ) : (
                  <XCircle className="text-red-400" size={14} />
                )}
                <span className="text-slate-400">Article 9: TPRM</span>
              </div>
              <div className="flex items-center gap-2">
                {doraReport.article10_compliant ? (
                  <CheckCircle className="text-emerald-400" size={14} />
                ) : (
                  <XCircle className="text-red-400" size={14} />
                )}
                <span className="text-slate-400">Article 10: Incidents</span>
              </div>
              <div className="flex items-center gap-2">
                {doraReport.article11_compliant ? (
                  <CheckCircle className="text-emerald-400" size={14} />
                ) : (
                  <XCircle className="text-red-400" size={14} />
                )}
                <span className="text-slate-400">Article 11: Resilience</span>
              </div>
            </div>
          </div>

          {/* NIS2 Compliance */}
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <BarChart3 className="text-emerald-400" size={20} />
              NIS2 Compliance
            </h3>
            <div className="text-4xl font-bold text-slate-100 mb-2">
              {nis2Report.overall_score.toFixed(1)}%
            </div>
            <div className="h-3 bg-slate-700 rounded-full overflow-hidden mb-4">
              <div
                className={`h-full ${
                  nis2Report.overall_score >= 90 ? "bg-emerald-500" :
                  nis2Report.overall_score >= 70 ? "bg-yellow-500" : "bg-red-500"
                }`}
                style={{ width: `${nis2Report.overall_score}%` }}
              />
            </div>
            <div className="space-y-1 text-xs">
              <div className="flex items-center gap-2">
                {nis2Report.article20_compliant ? (
                  <CheckCircle className="text-emerald-400" size={14} />
                ) : (
                  <XCircle className="text-red-400" size={14} />
                )}
                <span className="text-slate-400">Article 20: Management</span>
              </div>
              <div className="flex items-center gap-2">
                {nis2Report.article21_compliant ? (
                  <CheckCircle className="text-emerald-400" size={14} />
                ) : (
                  <XCircle className="text-red-400" size={14} />
                )}
                <span className="text-slate-400">Article 21: Baseline</span>
              </div>
              <div className="flex items-center gap-2">
                {nis2Report.article23_compliant ? (
                  <CheckCircle className="text-emerald-400" size={14} />
                ) : (
                  <XCircle className="text-red-400" size={14} />
                )}
                <span className="text-slate-400">Article 23: Reporting</span>
              </div>
            </div>
          </div>
        </div>

        {/* Executive Summary */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <FileText className="text-emerald-400" size={20} />
            Executive Summary
          </h2>
          <div className="prose prose-invert max-w-none">
            <p className="text-slate-300 leading-relaxed">
              {scorecard.executive_summary}
            </p>
          </div>
        </div>

        {/* Critical Issues */}
        {(scorecard.critical_issues_count > 0 || scorecard.high_risk_issues_count > 0) && (
          <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-red-400 mb-4 flex items-center gap-2">
              <AlertTriangle className="text-red-400" size={20} />
              Critical Issues Requiring Attention
            </h2>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-3xl font-bold text-red-400">{scorecard.critical_issues_count}</div>
                <div className="text-sm text-red-300">Critical Issues</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-orange-400">{scorecard.high_risk_issues_count}</div>
                <div className="text-sm text-orange-300">High Risk Issues</div>
              </div>
            </div>
          </div>
        )}

        {/* Recommendations */}
        {scorecard.recommendations.length > 0 && (
          <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
              <Target className="text-emerald-400" size={20} />
              Recommended Actions
            </h2>
            <ul className="space-y-2">
              {scorecard.recommendations.map((rec, idx) => (
                <li key={idx} className="flex items-start gap-2 text-slate-300">
                  <span className="text-emerald-400 mt-1">•</span>
                  <span>{rec}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Asset Compliance Overview */}
        <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-slate-100 mb-4 flex items-center gap-2">
            <BarChart3 className="text-emerald-400" size={20} />
            Asset Compliance Overview
          </h2>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <div className="text-2xl font-bold text-slate-100">{scorecard.total_assets}</div>
              <div className="text-sm text-slate-400">Total Assets</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-emerald-400">{scorecard.compliant_assets}</div>
              <div className="text-sm text-slate-400">Compliant</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-red-400">{scorecard.non_compliant_assets}</div>
              <div className="text-sm text-slate-400">Non-Compliant</div>
            </div>
          </div>
          <div className="h-3 bg-slate-700 rounded-full overflow-hidden mt-4">
            <div
              className="h-full bg-emerald-500"
              style={{ width: `${(scorecard.compliant_assets / scorecard.total_assets) * 100}%` }}
            />
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

