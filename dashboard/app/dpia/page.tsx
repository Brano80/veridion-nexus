"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { FileText, CheckCircle, Clock, AlertTriangle } from "lucide-react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchDPIAs() {
  const res = await fetch(`${API_BASE}/dpias`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch DPIAs: ${res.status}`);
  }
  return res.json();
}

export default function DPIAPage() {
  const { data: data, isLoading } = useQuery({
    queryKey: ["dpias"],
    queryFn: fetchDPIAs,
  });

  const dpias = data?.dpias || [];

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading...</div>
        </div>
      </DashboardLayout>
    );
  }

  const draftDPIAs = dpias.filter((d: any) => d.status === "DRAFT");
  const approvedDPIAs = dpias.filter((d: any) => d.status === "APPROVED");
  const highRiskDPIAs = dpias.filter((d: any) => d.risk_level === "HIGH");

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">DPIA Tracking</h1>
          <p className="text-slate-400">
            GDPR Article 35 - Data Protection Impact Assessment management
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-orange-900/20 border border-orange-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Clock className="text-orange-400" size={24} />
              <div className="text-3xl font-bold text-orange-400">
                {draftDPIAs.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Draft DPIAs</div>
          </div>
          <div className="p-6 bg-emerald-900/20 border border-emerald-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle className="text-emerald-400" size={24} />
              <div className="text-3xl font-bold text-emerald-400">
                {approvedDPIAs.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Approved</div>
          </div>
          <div className="p-6 bg-red-900/20 border border-red-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <AlertTriangle className="text-red-400" size={24} />
              <div className="text-3xl font-bold text-red-400">
                {highRiskDPIAs.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">High Risk</div>
          </div>
        </div>

        {/* DPIAs List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">All DPIAs</h2>
          <div className="space-y-3">
            {dpias.map((dpia: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-sm font-medium text-white">
                        {dpia.dpia_id}
                      </span>
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          dpia.status === "APPROVED"
                            ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                            : dpia.status === "DRAFT"
                            ? "bg-orange-900/30 text-orange-400 border border-orange-800"
                            : "bg-slate-800 text-slate-500 border border-slate-700"
                        }`}
                      >
                        {dpia.status}
                      </span>
                      <span
                        className={`px-2 py-1 rounded text-xs ${
                          dpia.risk_level === "HIGH"
                            ? "bg-red-900/30 text-red-400"
                            : "bg-orange-900/30 text-orange-400"
                        }`}
                      >
                        {dpia.risk_level}
                      </span>
                    </div>
                    <div className="text-sm text-slate-300 mb-1">
                      {dpia.activity_name}
                    </div>
                    <div className="text-xs text-slate-500">
                      Created: {dpia.created_at}
                      {dpia.consultation_required && (
                        <span className="ml-2 text-orange-400">
                          • Consultation Required
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {dpias.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              No DPIAs found
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

