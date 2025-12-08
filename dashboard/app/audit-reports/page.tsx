"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { FileText, Download, Calendar, CheckCircle, AlertTriangle } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function downloadReport(sealId?: string) {
  const url = sealId 
    ? `${API_BASE}/download_report?seal_id=${sealId}`
    : `${API_BASE}/download_report`;
  
  const response = await fetch(url, {
    headers: getAuthHeaders(),
  });
  if (!response.ok) {
    if (response.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error("Failed to download report");
  }
  
  const blob = await response.blob();
  const downloadUrl = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = downloadUrl;
  a.download = `annex-iv-report-${sealId || "all"}-${Date.now()}.pdf`;
  document.body.appendChild(a);
  a.click();
  window.URL.revokeObjectURL(downloadUrl);
  document.body.removeChild(a);
}

export default function AuditReportsPage() {
  const [selectedSealId, setSelectedSealId] = useState<string>("");

  const { data: logs, isLoading } = useQuery({
    queryKey: ["logs-for-reports"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE}/logs?limit=100`, {
        headers: getAuthHeaders(),
      });
      if (!response.ok) {
        if (response.status === 401) {
          throw new Error("Unauthorized - Please login");
        }
        throw new Error("Failed to fetch logs");
      }
      return response.json();
    },
    refetchInterval: 30000,
  });

  const records = logs?.data || [];
  const uniqueSealIds = Array.from(new Set(records.map((r: any) => r.seal_id).filter(Boolean)));

  const handleDownload = async () => {
    try {
      await downloadReport(selectedSealId || undefined);
    } catch (error) {
      alert("Failed to download report");
    }
  };

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading...</div>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
            <FileText className="text-emerald-400" size={32} />
            Audit & Annex IV Reports
          </h1>
          <p className="text-slate-400">
            Generate compliance reports and Annex IV technical documentation
          </p>
        </div>

        {/* Report Generation */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4 flex items-center gap-2">
            <Download size={20} />
            Generate Report
          </h2>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Select Seal ID (optional - leave empty for all records)
              </label>
              <select
                value={selectedSealId}
                onChange={(e) => setSelectedSealId(e.target.value)}
                className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:outline-none focus:border-emerald-500"
              >
                <option value="">All Records</option>
                {uniqueSealIds.map((sealId: string) => (
                  <option key={sealId} value={sealId}>
                    {sealId}
                  </option>
                ))}
              </select>
            </div>

            <button
              onClick={handleDownload}
              className="px-6 py-3 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg flex items-center gap-2 font-medium"
            >
              <Download size={18} />
              Download Annex IV Report (PDF)
            </button>
          </div>
        </div>

        {/* Report Information */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <FileText className="text-emerald-400" size={24} />
              <h3 className="text-lg font-semibold text-white">Annex IV Reports</h3>
            </div>
            <p className="text-slate-400 text-sm mb-4">
              Technical documentation required by EU AI Act Annex IV
            </p>
            <div className="flex items-center gap-2 text-emerald-400 text-sm">
              <CheckCircle size={16} />
              <span>Compliance Ready</span>
            </div>
          </div>

          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <Calendar className="text-blue-400" size={24} />
              <h3 className="text-lg font-semibold text-white">Audit Trail</h3>
            </div>
            <p className="text-slate-400 text-sm mb-4">
              Complete audit log of all compliance actions
            </p>
            <div className="text-slate-300 text-2xl font-bold">
              {records.length}
            </div>
          </div>

          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <AlertTriangle className="text-yellow-400" size={24} />
              <h3 className="text-lg font-semibold text-white">Compliance Status</h3>
            </div>
            <p className="text-slate-400 text-sm mb-4">
              Current compliance posture
            </p>
            <div className="flex items-center gap-2 text-emerald-400 text-sm">
              <CheckCircle size={16} />
              <span>All Systems Operational</span>
            </div>
          </div>
        </div>

        {/* Recent Records Preview */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Recent Compliance Records</h2>
          <div className="space-y-3">
            {records.slice(0, 10).map((record: any, i: number) => (
              <div
                key={i}
                className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-center gap-3">
                  <div className={`w-2 h-2 rounded-full ${
                    record.status?.includes("BLOCKED") ? "bg-red-500" :
                    record.status?.includes("APPROVED") ? "bg-emerald-500" :
                    "bg-yellow-500"
                  }`} />
                  <div>
                    <div className="text-sm font-medium text-white">
                      {record.action_summary}
                    </div>
                    <div className="text-xs text-slate-500">
                      {new Date(record.timestamp || record.created_at).toLocaleString()}
                    </div>
                  </div>
                </div>
                <span className={`px-2 py-1 rounded text-xs font-medium ${
                  record.status?.includes("BLOCKED") ? "bg-red-900/30 text-red-400" :
                  record.status?.includes("APPROVED") ? "bg-emerald-900/30 text-emerald-400" :
                  "bg-yellow-900/30 text-yellow-400"
                }`}>
                  {record.status}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

