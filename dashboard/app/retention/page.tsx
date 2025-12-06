"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Calendar, Clock, Trash2, AlertCircle } from "lucide-react";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchRetentionPolicies() {
  const res = await fetch(`${API_BASE}/retention/policies`);
  return res.json();
}

export default function RetentionPage() {
  const { data: data, isLoading } = useQuery({
    queryKey: ["retention-policies"],
    queryFn: fetchRetentionPolicies,
  });

  const policies = data?.policies || [];

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
            Retention Policies
          </h1>
          <p className="text-slate-400">
            GDPR Article 5(1)(e) - Storage limitation and automated deletion
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-blue-900/20 border border-blue-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Calendar className="text-blue-400" size={24} />
              <div className="text-3xl font-bold text-blue-400">
                {policies.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Active Policies</div>
          </div>
        </div>

        {/* Policies List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">
            Retention Policies
          </h2>
          <div className="space-y-3">
            {policies.map((policy: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-sm font-medium text-white">
                        {policy.policy_name}
                      </span>
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          policy.auto_delete
                            ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                            : "bg-slate-800 text-slate-500 border border-slate-700"
                        }`}
                      >
                        {policy.auto_delete ? "Auto-Delete" : "Manual"}
                      </span>
                    </div>
                    <div className="text-sm text-slate-300 mb-1">
                      Category: {policy.data_category}
                    </div>
                    <div className="text-xs text-slate-500">
                      Retention Period: {policy.retention_period_days} days
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {policies.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              No retention policies configured
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

