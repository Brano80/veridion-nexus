"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Shield, AlertTriangle, Clock, Users } from "lucide-react";
import { format } from "date-fns";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchBreaches() {
  const res = await fetch(`${API_BASE}/breaches`);
  return res.json();
}

export default function DataBreachesPage() {
  const queryClient = useQueryClient();

  const { data: breaches, isLoading } = useQuery({
    queryKey: ["data-breaches"],
    queryFn: fetchBreaches,
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

  const openBreaches = breaches?.filter((b: any) => b.status === "REPORTED");
  const totalAffected = breaches?.reduce(
    (sum: number, b: any) => sum + (b.affected_users?.length || 0),
    0
  );

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">
            Data Breach Management
          </h1>
          <p className="text-slate-400">
            GDPR Articles 33-34 - Data breach reporting and notification
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-red-900/20 border border-red-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <AlertTriangle className="text-red-400" size={24} />
              <div className="text-3xl font-bold text-red-400">
                {openBreaches?.length || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Open Breaches</div>
          </div>
          <div className="p-6 bg-orange-900/20 border border-orange-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Users className="text-orange-400" size={24} />
              <div className="text-3xl font-bold text-orange-400">
                {totalAffected || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Total Affected Users</div>
          </div>
          <div className="p-6 bg-blue-900/20 border border-blue-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Shield className="text-blue-400" size={24} />
              <div className="text-3xl font-bold text-blue-400">
                {breaches?.length || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Total Breaches</div>
          </div>
        </div>

        {/* Breaches List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">All Breaches</h2>
          <div className="space-y-3">
            {breaches?.map((breach: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="px-2 py-1 rounded text-xs font-medium bg-red-900/30 text-red-400 border border-red-800">
                        {breach.breach_type}
                      </span>
                      <span className="text-xs text-slate-500">
                        Detected: {breach.detected_at}
                      </span>
                    </div>
                    <div className="text-sm text-white mb-2">
                      {breach.description}
                    </div>
                    <div className="flex items-center gap-4 text-xs text-slate-400">
                      <span>
                        Affected Users: {breach.affected_users?.length || 0}
                      </span>
                      {breach.affected_records_count && (
                        <span>
                          Records: {breach.affected_records_count}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {breaches?.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              No data breaches reported
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

