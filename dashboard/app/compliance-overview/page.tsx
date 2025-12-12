"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Shield, FileText, TrendingUp, TrendingDown, Minus, Download, CheckCircle, XCircle, AlertCircle } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";
import { format } from "date-fns";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface ComplianceOverview {
  gdpr_score: number;
  eu_ai_act_score: number;
  overall_compliance_score: number;
  gdpr_articles: Array<{
    article_number: string;
    article_name: string;
    status: string;
    implementation_date: string | null;
    last_verified: string | null;
  }>;
  eu_ai_act_articles: Array<{
    article_number: string;
    article_name: string;
    status: string;
    implementation_date: string | null;
    last_verified: string | null;
  }>;
  last_updated: string;
}

interface MonthlySummary {
  month: string;
  gdpr_score: number;
  eu_ai_act_score: number;
  overall_score: number;
  total_requests: number;
  blocked_requests: number;
  compliance_violations: number;
  data_subject_requests: number;
  breach_notifications: number;
  human_oversight_reviews: number;
  risk_assessments: number;
  trends: {
    gdpr_trend: string;
    eu_ai_act_trend: string;
    violation_trend: string;
  };
}

async function fetchComplianceOverview(): Promise<ComplianceOverview> {
  const res = await fetch(`${API_BASE}/reports/compliance-overview`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

async function fetchMonthlySummary(month?: string): Promise<MonthlySummary> {
  const url = month 
    ? `${API_BASE}/reports/monthly-summary?month=${month}`
    : `${API_BASE}/reports/monthly-summary`;
  const res = await fetch(url, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

function getScoreColor(score: number) {
  if (score >= 90) return "text-emerald-400";
  if (score >= 75) return "text-yellow-400";
  if (score >= 50) return "text-orange-400";
  return "text-red-400";
}

function getScoreBgColor(score: number) {
  if (score >= 90) return "bg-emerald-500";
  if (score >= 75) return "bg-yellow-500";
  if (score >= 50) return "bg-orange-500";
  return "bg-red-500";
}

function getStatusIcon(status: string) {
  switch (status) {
    case "COMPLIANT":
      return <CheckCircle className="text-emerald-400" size={18} />;
    case "PARTIAL":
      return <AlertCircle className="text-yellow-400" size={18} />;
    case "NON_COMPLIANT":
      return <XCircle className="text-red-400" size={18} />;
    default:
      return <Minus className="text-slate-500" size={18} />;
  }
}

function getTrendIcon(trend: string) {
  switch (trend) {
    case "IMPROVING":
    case "DECREASING":
      return <TrendingUp className="text-emerald-400" size={16} />;
    case "DECLINING":
    case "INCREASING":
      return <TrendingDown className="text-red-400" size={16} />;
    default:
      return <Minus className="text-slate-500" size={16} />;
  }
}

export default function ComplianceOverviewPage() {
  const [selectedMonth, setSelectedMonth] = useState<string>("");

  const { data: overview, isLoading: loadingOverview } = useQuery({
    queryKey: ["compliance-overview"],
    queryFn: fetchComplianceOverview,
    refetchInterval: 300000, // 5 minutes
  });

  const { data: monthlySummary, isLoading: loadingMonthly } = useQuery({
    queryKey: ["monthly-summary", selectedMonth],
    queryFn: () => fetchMonthlySummary(selectedMonth || undefined),
    refetchInterval: 300000,
  });

  const handleExport = async (format: "csv" | "pdf") => {
    const month = selectedMonth || format(new Date(), "yyyy-MM");
    const url = `${API_BASE}/reports/monthly-summary/export?month=${month}&format=${format}`;
    window.open(url, "_blank");
  };

  if (loadingOverview || loadingMonthly) {
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

  if (!overview || !monthlySummary) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="text-slate-400">No compliance data available</div>
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
            <h1 className="text-3xl font-bold text-white mb-2">
              Compliance Overview
            </h1>
            <p className="text-slate-400">
              GDPR & EU AI Act compliance status and monthly summaries
            </p>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => handleExport("csv")}
              className="flex items-center gap-2 px-4 py-2 bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 rounded-lg transition-colors"
            >
              <Download size={18} />
              Export CSV
            </button>
            <button
              onClick={() => handleExport("pdf")}
              className="flex items-center gap-2 px-4 py-2 bg-red-900/50 hover:bg-red-800/80 text-red-400 border border-red-800 rounded-lg transition-colors"
            >
              <Download size={18} />
              Export PDF
            </button>
          </div>
        </div>

        {/* Overall Compliance Scores */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Overall Score */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
                <Shield className="text-emerald-400" size={20} />
                Overall Compliance
              </h3>
            </div>
            <div className="text-5xl font-bold mb-2" style={{ color: getScoreColor(overview.overall_compliance_score) }}>
              {overview.overall_compliance_score.toFixed(1)}%
            </div>
            <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
              <div
                className={`h-full ${getScoreBgColor(overview.overall_compliance_score)}`}
                style={{ width: `${overview.overall_compliance_score}%` }}
              />
            </div>
            <div className="text-xs text-slate-500 mt-2">
              Last updated: {format(new Date(overview.last_updated), "PPp")}
            </div>
          </div>

          {/* GDPR Score */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
                <FileText className="text-blue-400" size={20} />
                GDPR Compliance
              </h3>
            </div>
            <div className="text-5xl font-bold mb-2" style={{ color: getScoreColor(overview.gdpr_score) }}>
              {overview.gdpr_score.toFixed(1)}%
            </div>
            <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
              <div
                className={`h-full ${getScoreBgColor(overview.gdpr_score)}`}
                style={{ width: `${overview.gdpr_score}%` }}
              />
            </div>
            <div className="text-xs text-slate-500 mt-2">
              {overview.gdpr_articles.filter(a => a.status === "COMPLIANT").length} of {overview.gdpr_articles.length} articles compliant
            </div>
          </div>

          {/* EU AI Act Score */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
                <Shield className="text-purple-400" size={20} />
                EU AI Act Compliance
              </h3>
            </div>
            <div className="text-5xl font-bold mb-2" style={{ color: getScoreColor(overview.eu_ai_act_score) }}>
              {overview.eu_ai_act_score.toFixed(1)}%
            </div>
            <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
              <div
                className={`h-full ${getScoreBgColor(overview.eu_ai_act_score)}`}
                style={{ width: `${overview.eu_ai_act_score}%` }}
              />
            </div>
            <div className="text-xs text-slate-500 mt-2">
              {overview.eu_ai_act_articles.filter(a => a.status === "COMPLIANT").length} of {overview.eu_ai_act_articles.length} articles compliant
            </div>
          </div>
        </div>

        {/* Monthly Summary */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-white">Monthly Compliance Summary</h2>
            <input
              type="month"
              value={selectedMonth}
              onChange={(e) => setSelectedMonth(e.target.value)}
              className="px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
            />
          </div>
          
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400 mb-1">Total Requests</div>
              <div className="text-2xl font-bold text-white">{monthlySummary.total_requests.toLocaleString()}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400 mb-1">Blocked Requests</div>
              <div className="text-2xl font-bold text-red-400">{monthlySummary.blocked_requests.toLocaleString()}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400 mb-1">Data Subject Requests</div>
              <div className="text-2xl font-bold text-blue-400">{monthlySummary.data_subject_requests.toLocaleString()}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400 mb-1">Breach Notifications</div>
              <div className="text-2xl font-bold text-orange-400">{monthlySummary.breach_notifications.toLocaleString()}</div>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="text-sm text-slate-400">GDPR Trend</div>
                {getTrendIcon(monthlySummary.trends.gdpr_trend)}
              </div>
              <div className="text-lg font-semibold text-white">{monthlySummary.trends.gdpr_trend}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="text-sm text-slate-400">EU AI Act Trend</div>
                {getTrendIcon(monthlySummary.trends.eu_ai_act_trend)}
              </div>
              <div className="text-lg font-semibold text-white">{monthlySummary.trends.eu_ai_act_trend}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="text-sm text-slate-400">Violation Trend</div>
                {getTrendIcon(monthlySummary.trends.violation_trend)}
              </div>
              <div className="text-lg font-semibold text-white">{monthlySummary.trends.violation_trend}</div>
            </div>
          </div>
        </div>

        {/* GDPR Articles */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-white mb-4">GDPR Articles</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {overview.gdpr_articles.map((article, idx) => (
              <div
                key={idx}
                className="flex items-center justify-between p-3 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-center gap-3">
                  {getStatusIcon(article.status)}
                  <div>
                    <div className="font-semibold text-slate-200">{article.article_number}</div>
                    <div className="text-sm text-slate-400">{article.article_name}</div>
                  </div>
                </div>
                <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                  article.status === "COMPLIANT" 
                    ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                    : article.status === "PARTIAL"
                    ? "bg-yellow-900/30 text-yellow-400 border border-yellow-800"
                    : "bg-red-900/30 text-red-400 border border-red-800"
                }`}>
                  {article.status}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* EU AI Act Articles */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-white mb-4">EU AI Act Articles</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {overview.eu_ai_act_articles.map((article, idx) => (
              <div
                key={idx}
                className="flex items-center justify-between p-3 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-center gap-3">
                  {getStatusIcon(article.status)}
                  <div>
                    <div className="font-semibold text-slate-200">{article.article_number}</div>
                    <div className="text-sm text-slate-400">{article.article_name}</div>
                  </div>
                </div>
                <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                  article.status === "COMPLIANT" 
                    ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                    : article.status === "PARTIAL"
                    ? "bg-yellow-900/30 text-yellow-400 border border-yellow-800"
                    : "bg-red-900/30 text-red-400 border border-red-800"
                }`}>
                  {article.status}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

