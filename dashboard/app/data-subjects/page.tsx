"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Trash2, Search, AlertTriangle, CheckCircle } from "lucide-react";
import { useState } from "react";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchLogs() {
  const res = await fetch(`${API_BASE}/logs?limit=100`);
  return res.json();
}

async function shredData(sealId: string) {
  const res = await fetch(`${API_BASE}/shred_data`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ seal_id: sealId }),
  });
  if (!res.ok) {
    throw new Error("Failed to shred data");
  }
  return res.json();
}

export default function DataShreddingPage() {
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedSealId, setSelectedSealId] = useState("");
  const queryClient = useQueryClient();

  const { data: logsData, isLoading } = useQuery({
    queryKey: ["logs-for-shredding"],
    queryFn: fetchLogs,
    refetchInterval: 10000,
  });

  const shredMutation = useMutation({
    mutationFn: shredData,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["logs-for-shredding"] });
      alert("Data successfully shredded (crypto-shredded)");
      setSelectedSealId("");
    },
    onError: () => {
      alert("Failed to shred data");
    },
  });

  const logs = logsData?.logs || [];
  const filteredLogs = logs.filter((log: any) => {
    if (!searchTerm) return true;
    const term = searchTerm.toLowerCase();
    return (
      log.seal_id?.toLowerCase().includes(term) ||
      log.user_id?.toLowerCase().includes(term) ||
      log.agent_id?.toLowerCase().includes(term)
    );
  });

  const handleShred = () => {
    if (!selectedSealId) {
      alert("Please select a record to shred");
      return;
    }
    if (confirm(`Are you sure you want to crypto-shred data for seal ID: ${selectedSealId}? This action cannot be undone.`)) {
      shredMutation.mutate(selectedSealId);
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
            <Trash2 className="text-red-400" size={32} />
            Data Shredding (GDPR Article 17)
          </h1>
          <p className="text-slate-400">
            Crypto-shredding for Right to be Forgotten - GDPR compliant data erasure
          </p>
        </div>

        {/* Warning */}
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-4 flex items-start gap-3">
          <AlertTriangle className="text-red-400 flex-shrink-0 mt-0.5" size={20} />
          <div>
            <h3 className="text-red-400 font-semibold mb-1">Irreversible Action</h3>
            <p className="text-slate-300 text-sm">
              Crypto-shredding permanently encrypts data with a destroyed key, making it impossible to recover.
              This action complies with GDPR Article 17 (Right to be Forgotten).
            </p>
          </div>
        </div>

        {/* Shredding Action */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Crypto-Shred Data</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Select Seal ID to Shred
              </label>
              <input
                type="text"
                value={selectedSealId}
                onChange={(e) => setSelectedSealId(e.target.value)}
                placeholder="Enter seal ID..."
                className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-red-500"
              />
            </div>
            <button
              onClick={handleShred}
              disabled={!selectedSealId || shredMutation.isPending}
              className="px-6 py-3 bg-red-600 hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg flex items-center gap-2 font-medium"
            >
              <Trash2 size={18} />
              {shredMutation.isPending ? "Shredding..." : "Crypto-Shred Data"}
            </button>
          </div>
        </div>

        {/* Records List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-bold text-white">Compliance Records</h2>
            <div className="flex-1 max-w-md relative ml-4">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-500" size={18} />
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Search by seal ID, user ID, or agent ID..."
                className="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-emerald-500"
              />
            </div>
          </div>

          <div className="space-y-2 max-h-96 overflow-y-auto">
            {filteredLogs.map((log: any, i: number) => (
              <div
                key={i}
                className={`p-4 rounded-lg border cursor-pointer transition-colors ${
                  selectedSealId === log.seal_id
                    ? "bg-red-900/20 border-red-800"
                    : "bg-slate-800/50 border-slate-700 hover:bg-slate-800"
                }`}
                onClick={() => setSelectedSealId(log.seal_id)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <div className="text-sm font-medium text-white mb-1">
                      {log.action_summary}
                    </div>
                    <div className="text-xs text-slate-500 font-mono">
                      Seal ID: {log.seal_id} • User: {log.user_id || "N/A"} • Agent: {log.agent_id}
                    </div>
                    <div className="text-xs text-slate-500 mt-1">
                      {new Date(log.timestamp || log.created_at).toLocaleString()}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
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
                    {selectedSealId === log.seal_id && (
                      <CheckCircle className="text-red-400" size={18} />
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {filteredLogs.length === 0 && (
            <div className="text-center py-8 text-slate-500">
              No records found
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}
