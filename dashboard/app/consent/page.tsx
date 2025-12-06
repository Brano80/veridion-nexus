"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { ClipboardCheck, CheckCircle, XCircle, Clock } from "lucide-react";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchConsents() {
  // This would need a new endpoint to list all consents
  // For now, we'll show a placeholder
  return [];
}

export default function ConsentPage() {
  const { data: consents, isLoading } = useQuery({
    queryKey: ["consents"],
    queryFn: fetchConsents,
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

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">
            Consent Management
          </h1>
          <p className="text-slate-400">
            GDPR Articles 6, 7 - Consent tracking and management
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-emerald-900/20 border border-emerald-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle className="text-emerald-400" size={24} />
              <div className="text-3xl font-bold text-emerald-400">0</div>
            </div>
            <div className="text-sm text-slate-400">Active Consents</div>
          </div>
          <div className="p-6 bg-red-900/20 border border-red-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <XCircle className="text-red-400" size={24} />
              <div className="text-3xl font-bold text-red-400">0</div>
            </div>
            <div className="text-sm text-slate-400">Withdrawn</div>
          </div>
          <div className="p-6 bg-blue-900/20 border border-blue-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <ClipboardCheck className="text-blue-400" size={24} />
              <div className="text-3xl font-bold text-blue-400">0</div>
            </div>
            <div className="text-sm text-slate-400">Total Consents</div>
          </div>
        </div>

        {/* Info Message */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <div className="text-center py-12 text-slate-500">
            <ClipboardCheck size={48} className="mx-auto mb-4 opacity-50" />
            <p>Consent management interface</p>
            <p className="text-sm mt-2">
              Use the API to manage consents programmatically
            </p>
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

