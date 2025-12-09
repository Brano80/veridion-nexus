"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { ScrollText, Search, Download, Filter } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchLogs(page: number = 1, limit: number = 50) {
  const response = await fetch(`${API_BASE}/logs?page=${page}&limit=${limit}`, {
    headers: getAuthHeaders(),
  });
  if (!response.ok) {
    if (response.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch logs: ${response.status}`);
  }
  return response.json();
}

export default function RuntimeLogsPage() {
  const [page, setPage] = useState(1);
  const [searchTerm, setSearchTerm] = useState("");
  const limit = 50;

  const { data, isLoading, error } = useQuery({
    queryKey: ["logs", page],
    queryFn: () => fetchLogs(page, limit),
    refetchInterval: 10000, // Refresh every 10 seconds
  });

  const logs = data?.data || [];
  const totalPages = data?.pagination?.total_pages || 1;

  const filteredLogs = logs.filter((log: any) => {
    if (!searchTerm) return true;
    const term = searchTerm.toLowerCase();
    return (
      log.action_summary?.toLowerCase().includes(term) ||
      log.agent_id?.toLowerCase().includes(term) ||
      log.seal_id?.toLowerCase().includes(term) ||
      log.status?.toLowerCase().includes(term)
    );
  });

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading logs...</div>
        </div>
      </DashboardLayout>
    );
  }

  if (error) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-center">
            <div className="text-red-400 mb-4">Error loading logs</div>
            <div className="text-slate-400 text-sm mb-4">
              {error instanceof Error ? error.message : "Unknown error"}
            </div>
            {error instanceof Error && error.message.includes("Unauthorized") && (
              <a
                href="/login"
                className="text-emerald-400 hover:text-emerald-300 underline"
              >
                Please login to continue
              </a>
            )}
          </div>
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
            <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
              <ScrollText className="text-emerald-400" size={32} />
              Runtime Logs Explorer
            </h1>
            <p className="text-slate-400">
              Real-time compliance audit trail and runtime logs
            </p>
          </div>
          <button className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg flex items-center gap-2">
            <Download size={18} />
            Export
          </button>
        </div>

        {/* Search and Filters */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-4">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-500" size={18} />
              <input
                type="text"
                placeholder="Search logs by action, agent, seal ID, or status..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-emerald-500"
              />
            </div>
            <button className="px-4 py-2 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg flex items-center gap-2 text-slate-300">
              <Filter size={18} />
              Filters
            </button>
          </div>
        </div>

        {/* Logs Table */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-slate-800 border-b border-slate-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Timestamp
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Agent ID
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Action
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Seal ID
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Risk Level
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider">
                    Region
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800">
                {filteredLogs.map((log: any, i: number) => (
                  <tr key={i} className="hover:bg-slate-800/50 transition-colors">
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-300">
                      {new Date(log.timestamp || log.created_at).toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-300 font-mono">
                      {log.agent_id}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-300">
                      {log.action_summary}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          log.status?.includes("BLOCKED") || log.status?.includes("REJECTED")
                            ? "bg-red-900/30 text-red-400"
                            : log.status?.includes("APPROVED") || log.status?.includes("ALLOWED")
                            ? "bg-emerald-900/30 text-emerald-400"
                            : "bg-yellow-900/30 text-yellow-400"
                        }`}
                      >
                        {log.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-400 font-mono">
                      {log.seal_id?.substring(0, 16)}...
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {log.risk_level && (
                        <span
                          className={`px-2 py-1 rounded text-xs font-medium ${
                            log.risk_level === "HIGH"
                              ? "bg-red-900/30 text-red-400"
                              : log.risk_level === "MEDIUM"
                              ? "bg-yellow-900/30 text-yellow-400"
                              : "bg-emerald-900/30 text-emerald-400"
                          }`}
                        >
                          {log.risk_level}
                        </span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-400 font-mono">
                      {log.target_region || "Unknown"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="px-6 py-4 border-t border-slate-800 flex items-center justify-between">
              <div className="text-sm text-slate-400">
                Page {page} of {totalPages}
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                  disabled={page === 1}
                  className="px-4 py-2 bg-slate-800 hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed border border-slate-700 rounded-lg text-slate-300"
                >
                  Previous
                </button>
                <button
                  onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                  disabled={page === totalPages}
                  className="px-4 py-2 bg-slate-800 hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed border border-slate-700 rounded-lg text-slate-300"
                >
                  Next
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

