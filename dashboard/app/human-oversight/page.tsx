"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CheckCircle, XCircle, Clock, User } from "lucide-react";
import { useState } from "react";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchPendingOversight() {
  // Fetch compliance records that require human oversight
  const res = await fetch(`${API_BASE}/logs`);
  const records = await res.json();
  return records.filter(
    (r: any) => r.human_oversight_status === "PENDING"
  );
}

async function approveAction(sealId: string, reviewerId: string, comments: string) {
  const res = await fetch(`${API_BASE}/action/${sealId}/approve`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ reviewer_id: reviewerId, comments }),
  });
  return res.json();
}

async function rejectAction(sealId: string, reviewerId: string, comments: string) {
  const res = await fetch(`${API_BASE}/action/${sealId}/reject`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ reviewer_id: reviewerId, comments }),
  });
  return res.json();
}

export default function HumanOversightPage() {
  const queryClient = useQueryClient();
  const [selectedRecord, setSelectedRecord] = useState<any>(null);
  const [reviewerId, setReviewerId] = useState("admin-001");
  const [comments, setComments] = useState("");

  const { data: pendingRecords, isLoading } = useQuery({
    queryKey: ["pending-oversight"],
    queryFn: fetchPendingOversight,
  });

  const approveMutation = useMutation({
    mutationFn: ({ sealId }: { sealId: string }) =>
      approveAction(sealId, reviewerId, comments),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["pending-oversight"] });
      setSelectedRecord(null);
      setComments("");
    },
  });

  const rejectMutation = useMutation({
    mutationFn: ({ sealId }: { sealId: string }) =>
      rejectAction(sealId, reviewerId, comments),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["pending-oversight"] });
      setSelectedRecord(null);
      setComments("");
    },
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
            Human Oversight Queue
          </h1>
          <p className="text-slate-400">
            Review and approve/reject AI actions requiring human oversight
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-orange-900/20 border border-orange-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Clock className="text-orange-400" size={24} />
              <div className="text-3xl font-bold text-orange-400">
                {pendingRecords?.length || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Pending Reviews</div>
          </div>
        </div>

        {/* Pending Items */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Pending Actions</h2>
          <div className="space-y-3">
            {pendingRecords?.map((record: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700 hover:border-orange-600 transition-colors cursor-pointer"
                onClick={() => setSelectedRecord(record)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <div className="text-sm font-medium text-white mb-1">
                      {record.action_summary}
                    </div>
                    <div className="text-xs text-slate-500">
                      Seal ID: {record.seal_id} • {record.timestamp}
                    </div>
                    {record.risk_level && (
                      <div className="mt-2">
                        <span
                          className={`text-xs px-2 py-1 rounded ${
                            record.risk_level === "HIGH"
                              ? "bg-red-900/30 text-red-400"
                              : "bg-orange-900/30 text-orange-400"
                          }`}
                        >
                          Risk: {record.risk_level}
                        </span>
                      </div>
                    )}
                  </div>
                  <Clock className="text-orange-400" size={20} />
                </div>
              </div>
            ))}
          </div>

          {pendingRecords?.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              No pending actions requiring human oversight
            </div>
          )}
        </div>

        {/* Review Modal */}
        {selectedRecord && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
            <div className="bg-slate-900 border border-slate-800 rounded-lg p-6 max-w-2xl w-full">
              <h3 className="text-xl font-bold text-white mb-4">
                Review Action
              </h3>
              <div className="space-y-4 mb-6">
                <div>
                  <label className="text-sm text-slate-400">Action</label>
                  <div className="text-white mt-1">{selectedRecord.action_summary}</div>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Seal ID</label>
                  <div className="text-slate-300 font-mono text-sm mt-1">
                    {selectedRecord.seal_id}
                  </div>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Risk Level</label>
                  <div className="text-white mt-1">{selectedRecord.risk_level || "N/A"}</div>
                </div>
                <div>
                  <label className="text-sm text-slate-400 mb-2 block">
                    Reviewer ID
                  </label>
                  <input
                    type="text"
                    value={reviewerId}
                    onChange={(e) => setReviewerId(e.target.value)}
                    className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
                  />
                </div>
                <div>
                  <label className="text-sm text-slate-400 mb-2 block">
                    Comments
                  </label>
                  <textarea
                    value={comments}
                    onChange={(e) => setComments(e.target.value)}
                    rows={3}
                    className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
                    placeholder="Add review comments..."
                  />
                </div>
              </div>
              <div className="flex gap-3 justify-end">
                <button
                  onClick={() => {
                    setSelectedRecord(null);
                    setComments("");
                  }}
                  className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={() =>
                    rejectMutation.mutate({ sealId: selectedRecord.seal_id })
                  }
                  disabled={rejectMutation.isPending}
                  className="px-4 py-2 bg-red-900/50 hover:bg-red-800/80 text-red-400 border border-red-800 rounded-lg transition-colors flex items-center gap-2"
                >
                  <XCircle size={18} />
                  Reject
                </button>
                <button
                  onClick={() =>
                    approveMutation.mutate({ sealId: selectedRecord.seal_id })
                  }
                  disabled={approveMutation.isPending}
                  className="px-4 py-2 bg-emerald-900/50 hover:bg-emerald-800/80 text-emerald-400 border border-emerald-800 rounded-lg transition-colors flex items-center gap-2"
                >
                  <CheckCircle size={18} />
                  Approve
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

