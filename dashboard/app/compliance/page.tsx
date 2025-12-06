"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { format } from "date-fns";
import { Trash2, Download, Search, Filter } from "lucide-react";
import { useState } from "react";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchComplianceRecords() {
  const res = await fetch(`${API_BASE}/logs`);
  return res.json();
}

async function shredData(sealId: string) {
  const res = await fetch(`${API_BASE}/shred_data`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ seal_id: sealId }),
  });
  return res.json();
}

export default function CompliancePage() {
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");

  const { data: records, isLoading, refetch } = useQuery({
    queryKey: ["compliance-records"],
    queryFn: fetchComplianceRecords,
  });

  const handleShred = async (sealId: string) => {
    if (!confirm("GDPR REQUEST: Permanently shred data for this transaction?")) {
      return;
    }
    await shredData(sealId);
    refetch();
  };

  const filteredRecords = records?.filter((record: any) => {
    const matchesSearch =
      record.action_summary?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      record.seal_id?.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesStatus =
      statusFilter === "all" || record.status?.includes(statusFilter.toUpperCase());
    return matchesSearch && matchesStatus;
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
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">
              Compliance Records
            </h1>
            <p className="text-slate-400">
              All compliance actions and eIDAS seals
            </p>
          </div>
          <button
            onClick={() => window.open(`${API_BASE}/download_report`, "_blank")}
            className="flex items-center gap-2 px-4 py-2 bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 rounded-lg transition-colors"
          >
            <Download size={18} />
            Download Annex IV
          </button>
        </div>

        {/* Filters */}
        <div className="flex gap-4 items-center">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-500" size={18} />
            <input
              type="text"
              placeholder="Search by action or seal ID..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-slate-900 border border-slate-800 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-emerald-600"
            />
          </div>
          <div className="flex items-center gap-2">
            <Filter size={18} className="text-slate-500" />
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="px-4 py-2 bg-slate-900 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-600"
            >
              <option value="all">All Status</option>
              <option value="sealed">Sealed</option>
              <option value="blocked">Blocked</option>
              <option value="erased">Erased</option>
            </select>
          </div>
        </div>

        {/* Table */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-slate-950 border-b border-slate-800">
                <tr>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Timestamp
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Action
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Seal ID
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Risk Level
                  </th>
                  <th className="px-6 py-4 text-right text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800">
                {filteredRecords?.map((record: any, i: number) => (
                  <tr
                    key={i}
                    className="hover:bg-slate-800/50 transition-colors"
                  >
                    <td className="px-6 py-4 text-sm font-mono text-slate-400">
                      {record.timestamp}
                    </td>
                    <td className="px-6 py-4">
                      <div
                        className={`text-sm font-medium ${
                          record.status?.includes("ERASED")
                            ? "text-slate-600 line-through italic"
                            : "text-white"
                        }`}
                      >
                        {record.action_summary}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-xs font-mono text-slate-500">
                        {record.seal_id}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                          record.status?.includes("BLOCKED") ||
                          record.status === "REVOKED"
                            ? "bg-red-950 text-red-400 border border-red-900"
                            : record.status?.includes("ERASED")
                            ? "bg-slate-800 text-slate-500 border border-slate-600"
                            : "bg-emerald-950 text-emerald-400 border border-emerald-900"
                        }`}
                      >
                        {record.status}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={`text-xs font-medium ${
                          record.risk_level === "HIGH"
                            ? "text-red-400"
                            : record.risk_level === "MEDIUM"
                            ? "text-orange-400"
                            : "text-slate-400"
                        }`}
                      >
                        {record.risk_level || "N/A"}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right">
                      {!record.status?.includes("ERASED") &&
                        !record.status?.includes("BLOCKED") && (
                          <button
                            onClick={() => handleShred(record.seal_id)}
                            className="p-2 text-slate-500 hover:text-red-400 hover:bg-red-900/20 rounded transition-colors"
                            title="Crypto-Shred this record"
                          >
                            <Trash2 size={16} />
                          </button>
                        )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {filteredRecords?.length === 0 && (
          <div className="text-center py-12 text-slate-500">
            No records found matching your criteria
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

