"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Ban, CheckCircle, XCircle, Plus, Search, AlertCircle, Clock } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface ProcessingObjection {
  objection_id: string;
  user_id: string;
  objection_type: string;
  status: string;
  requested_at: string;
  rejection_reason?: string;
}

interface ObjectionsResponse {
  user_id: string;
  objections: ProcessingObjection[];
}

async function fetchObjections(userId: string): Promise<ObjectionsResponse> {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/objections`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    throw new Error(`Failed to fetch objections: ${res.status}`);
  }
  return res.json();
}

async function createObjection(userId: string, objection: any) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/object`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify(objection),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to create objection");
  }
  return res.json();
}

async function withdrawObjection(userId: string, reason?: string) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/withdraw_objection`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      user_id: userId,
      reason: reason || "Withdrawn via dashboard",
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to withdraw objection");
  }
  return res.json();
}

async function rejectObjection(userId: string, rejectionReason: string) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/reject_objection`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      user_id: userId,
      rejection_reason: rejectionReason,
      rejected_by: "admin",
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to reject objection");
  }
  return res.json();
}

export default function ProcessingObjectionsPage() {
  const [searchUserId, setSearchUserId] = useState("");
  const [selectedUserId, setSelectedUserId] = useState("");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showWithdrawModal, setShowWithdrawModal] = useState(false);
  const [showRejectModal, setShowRejectModal] = useState(false);
  const [objectionType, setObjectionType] = useState("FULL");
  const [objectedActions, setObjectedActions] = useState("");
  const [legalBasis, setLegalBasis] = useState("");
  const [reason, setReason] = useState("");
  const [rejectionReason, setRejectionReason] = useState("");
  const queryClient = useQueryClient();

  const { data: objectionsData, isLoading, error } = useQuery({
    queryKey: ["objections", selectedUserId],
    queryFn: () => fetchObjections(selectedUserId),
    enabled: !!selectedUserId,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createObjection(selectedUserId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["objections", selectedUserId] });
      setShowCreateModal(false);
      setObjectionType("FULL");
      setObjectedActions("");
      setLegalBasis("");
      setReason("");
      alert("Processing objection created successfully");
    },
    onError: (err: any) => {
      alert(`Failed to create objection: ${err.message}`);
    },
  });

  const withdrawMutation = useMutation({
    mutationFn: () => withdrawObjection(selectedUserId, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["objections", selectedUserId] });
      setShowWithdrawModal(false);
      setReason("");
      alert("Processing objection withdrawn successfully");
    },
    onError: (err: any) => {
      alert(`Failed to withdraw objection: ${err.message}`);
    },
  });

  const rejectMutation = useMutation({
    mutationFn: () => rejectObjection(selectedUserId, rejectionReason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["objections", selectedUserId] });
      setShowRejectModal(false);
      setRejectionReason("");
      alert("Processing objection rejected successfully");
    },
    onError: (err: any) => {
      alert(`Failed to reject objection: ${err.message}`);
    },
  });

  const handleSearch = () => {
    if (!searchUserId.trim()) {
      alert("Please enter a user ID");
      return;
    }
    setSelectedUserId(searchUserId.trim());
  };

  const handleCreate = () => {
    if (!selectedUserId) {
      alert("Please search for a user first");
      return;
    }

    if (!rejectionReason && objectionType === "REJECTED") {
      alert("Rejection reason is required per GDPR Article 21(1)");
      return;
    }

    const objection: any = {
      user_id: selectedUserId,
      objection_type: objectionType,
      reason: reason || undefined,
    };

    if (objectionType === "PARTIAL" || objectionType === "SPECIFIC_ACTION") {
      const actions = objectedActions
        .split(",")
        .map((a) => a.trim())
        .filter((a) => a.length > 0);
      if (actions.length === 0) {
        alert("Please specify at least one objected action for PARTIAL/SPECIFIC_ACTION type");
        return;
      }
      objection.objected_actions = actions;
    }

    if (legalBasis) {
      objection.legal_basis = legalBasis;
    }

    createMutation.mutate(objection);
  };

  const objections = objectionsData?.objections || [];
  const activeObjection = objections.find((o) => o.status === "ACTIVE");

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
            <Ban className="text-red-400" size={32} />
            Processing Objections
          </h1>
          <p className="text-slate-400">
            GDPR Article 21 - Right to Object
          </p>
        </div>

        {/* Search User */}
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm font-medium text-slate-300 mb-2">
                User ID
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={searchUserId}
                  onChange={(e) => setSearchUserId(e.target.value)}
                  onKeyPress={(e) => e.key === "Enter" && handleSearch()}
                  placeholder="Enter user ID to view objections"
                  className="flex-1 px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                />
                <button
                  onClick={handleSearch}
                  className="px-6 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
                >
                  <Search size={18} />
                  Search
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Active Objection Alert */}
        {activeObjection && (
          <div className="bg-red-900/30 border border-red-700 rounded-lg p-4 flex items-center gap-3">
            <AlertCircle className="text-red-400" size={24} />
            <div className="flex-1">
              <p className="text-red-400 font-medium">
                Active Objection Found
              </p>
              <p className="text-slate-400 text-sm">
                Type: {activeObjection.objection_type} | Requested:{" "}
                {new Date(activeObjection.requested_at).toLocaleString()}
              </p>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => setShowWithdrawModal(true)}
                className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
              >
                <CheckCircle size={16} />
                Withdraw
              </button>
              <button
                onClick={() => setShowRejectModal(true)}
                className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
              >
                <XCircle size={16} />
                Reject
              </button>
            </div>
          </div>
        )}

        {/* Objections List */}
        {selectedUserId && (
          <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-white">
                Objections for: {selectedUserId}
              </h2>
              {!activeObjection && (
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
                >
                  <Plus size={18} />
                  Create Objection
                </button>
              )}
            </div>

            {isLoading ? (
              <div className="text-center py-8 text-slate-400">Loading...</div>
            ) : error ? (
              <div className="text-center py-8 text-red-400">
                Error: {(error as Error).message}
              </div>
            ) : objections.length === 0 ? (
              <div className="text-center py-8 text-slate-400">
                No objections found for this user
              </div>
            ) : (
              <div className="space-y-3">
                {objections.map((objection) => (
                  <div
                    key={objection.objection_id}
                    className="bg-slate-700/50 border border-slate-600 rounded-lg p-4"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <span
                            className={`px-3 py-1 rounded-full text-xs font-medium ${
                              objection.status === "ACTIVE"
                                ? "bg-red-900/50 text-red-400"
                                : objection.status === "WITHDRAWN"
                                ? "bg-green-900/50 text-green-400"
                                : objection.status === "REJECTED"
                                ? "bg-orange-900/50 text-orange-400"
                                : "bg-slate-600 text-slate-400"
                            }`}
                          >
                            {objection.status}
                          </span>
                          <span className="text-sm text-slate-400">
                            {objection.objection_type}
                          </span>
                        </div>
                        <p className="text-sm text-slate-300 font-mono mb-1">
                          {objection.objection_id}
                        </p>
                        <p className="text-xs text-slate-500">
                          Requested: {new Date(objection.requested_at).toLocaleString()}
                        </p>
                        {objection.rejection_reason && (
                          <div className="mt-2 p-2 bg-orange-900/20 border border-orange-700/50 rounded text-xs text-orange-400">
                            <strong>Rejection Reason:</strong> {objection.rejection_reason}
                          </div>
                        )}
                      </div>
                      {objection.status === "ACTIVE" && (
                        <Ban className="text-red-400" size={20} />
                      )}
                      {objection.status === "WITHDRAWN" && (
                        <CheckCircle className="text-green-400" size={20} />
                      )}
                      {objection.status === "REJECTED" && (
                        <XCircle className="text-orange-400" size={20} />
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Create Objection Modal */}
        {showCreateModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Create Processing Objection
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Objection Type
                  </label>
                  <select
                    value={objectionType}
                    onChange={(e) => setObjectionType(e.target.value)}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  >
                    <option value="FULL">FULL - Object to all processing</option>
                    <option value="PARTIAL">PARTIAL - Object to specific actions</option>
                    <option value="SPECIFIC_ACTION">SPECIFIC_ACTION - Object to specific actions</option>
                    <option value="DIRECT_MARKETING">DIRECT_MARKETING - Object to direct marketing</option>
                    <option value="PROFILING">PROFILING - Object to profiling</option>
                  </select>
                </div>

                {(objectionType === "PARTIAL" ||
                  objectionType === "SPECIFIC_ACTION") && (
                  <div>
                    <label className="block text-sm font-medium text-slate-300 mb-2">
                      Objected Actions (comma-separated)
                    </label>
                    <input
                      type="text"
                      value={objectedActions}
                      onChange={(e) => setObjectedActions(e.target.value)}
                      placeholder="credit_scoring, automated_decision"
                      className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                    />
                  </div>
                )}

                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Legal Basis (optional)
                  </label>
                  <input
                    type="text"
                    value={legalBasis}
                    onChange={(e) => setLegalBasis(e.target.value)}
                    placeholder="LEGITIMATE_INTERESTS, PUBLIC_TASK"
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="Reason for objection"
                    rows={3}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowCreateModal(false)}
                    className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleCreate}
                    disabled={createMutation.isPending}
                    className="flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                  >
                    {createMutation.isPending ? "Creating..." : "Create"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Withdraw Objection Modal */}
        {showWithdrawModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Withdraw Processing Objection
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="Reason for withdrawing objection"
                    rows={3}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowWithdrawModal(false)}
                    className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={() => withdrawMutation.mutate()}
                    disabled={withdrawMutation.isPending}
                    className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                  >
                    {withdrawMutation.isPending ? "Withdrawing..." : "Withdraw Objection"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Reject Objection Modal */}
        {showRejectModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Reject Processing Objection
              </h3>
              <div className="space-y-4">
                <div className="bg-orange-900/20 border border-orange-700/50 rounded-lg p-3 mb-4">
                  <p className="text-xs text-orange-400">
                    <strong>GDPR Article 21(1):</strong> Rejection reason is required when rejecting an objection.
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Rejection Reason <span className="text-red-400">*</span>
                  </label>
                  <textarea
                    value={rejectionReason}
                    onChange={(e) => setRejectionReason(e.target.value)}
                    placeholder="Processing is necessary for the performance of a task carried out in the public interest..."
                    rows={4}
                    required
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowRejectModal(false)}
                    className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={() => rejectMutation.mutate()}
                    disabled={rejectMutation.isPending || !rejectionReason.trim()}
                    className="flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                  >
                    {rejectMutation.isPending ? "Rejecting..." : "Reject Objection"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

