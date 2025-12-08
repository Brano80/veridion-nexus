"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Lock, Unlock, Plus, Search, AlertCircle, CheckCircle, Clock } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface ProcessingRestriction {
  restriction_id: string;
  user_id: string;
  restriction_type: string;
  status: string;
  requested_at: string;
  expires_at?: string;
}

interface RestrictionsResponse {
  user_id: string;
  restrictions: ProcessingRestriction[];
}

async function fetchRestrictions(userId: string): Promise<RestrictionsResponse> {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/restrictions`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    throw new Error(`Failed to fetch restrictions: ${res.status}`);
  }
  return res.json();
}

async function createRestriction(userId: string, restriction: any) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/restrict`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify(restriction),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to create restriction");
  }
  return res.json();
}

async function liftRestriction(userId: string, reason?: string) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/lift_restriction`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      user_id: userId,
      reason: reason || "Lifted via dashboard",
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to lift restriction");
  }
  return res.json();
}

export default function ProcessingRestrictionsPage() {
  const [searchUserId, setSearchUserId] = useState("");
  const [selectedUserId, setSelectedUserId] = useState("");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showLiftModal, setShowLiftModal] = useState(false);
  const [restrictionType, setRestrictionType] = useState("FULL");
  const [restrictedActions, setRestrictedActions] = useState("");
  const [reason, setReason] = useState("");
  const [expiresAt, setExpiresAt] = useState("");
  const queryClient = useQueryClient();

  const { data: restrictionsData, isLoading, error } = useQuery({
    queryKey: ["restrictions", selectedUserId],
    queryFn: () => fetchRestrictions(selectedUserId),
    enabled: !!selectedUserId,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createRestriction(selectedUserId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["restrictions", selectedUserId] });
      setShowCreateModal(false);
      setRestrictionType("FULL");
      setRestrictedActions("");
      setReason("");
      setExpiresAt("");
      alert("Processing restriction created successfully");
    },
    onError: (err: any) => {
      alert(`Failed to create restriction: ${err.message}`);
    },
  });

  const liftMutation = useMutation({
    mutationFn: () => liftRestriction(selectedUserId, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["restrictions", selectedUserId] });
      setShowLiftModal(false);
      setReason("");
      alert("Processing restriction lifted successfully");
    },
    onError: (err: any) => {
      alert(`Failed to lift restriction: ${err.message}`);
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

    const restriction: any = {
      user_id: selectedUserId,
      restriction_type: restrictionType,
      reason: reason || undefined,
    };

    if (restrictionType === "PARTIAL" || restrictionType === "SPECIFIC_ACTION") {
      const actions = restrictedActions
        .split(",")
        .map((a) => a.trim())
        .filter((a) => a.length > 0);
      if (actions.length === 0) {
        alert("Please specify at least one restricted action for PARTIAL/SPECIFIC_ACTION type");
        return;
      }
      restriction.restricted_actions = actions;
    }

    if (expiresAt) {
      restriction.expires_at = expiresAt;
    }

    createMutation.mutate(restriction);
  };

  const restrictions = restrictionsData?.restrictions || [];
  const activeRestriction = restrictions.find((r) => r.status === "ACTIVE");

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
            <Lock className="text-orange-400" size={32} />
            Processing Restrictions
          </h1>
          <p className="text-slate-400">
            GDPR Article 18 - Right to Restriction of Processing
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
                  placeholder="Enter user ID to view restrictions"
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

        {/* Active Restriction Alert */}
        {activeRestriction && (
          <div className="bg-orange-900/30 border border-orange-700 rounded-lg p-4 flex items-center gap-3">
            <AlertCircle className="text-orange-400" size={24} />
            <div className="flex-1">
              <p className="text-orange-400 font-medium">
                Active Restriction Found
              </p>
              <p className="text-slate-400 text-sm">
                Type: {activeRestriction.restriction_type} | Requested:{" "}
                {new Date(activeRestriction.requested_at).toLocaleString()}
              </p>
            </div>
            <button
              onClick={() => setShowLiftModal(true)}
              className="px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
            >
              <Unlock size={16} />
              Lift Restriction
            </button>
          </div>
        )}

        {/* Restrictions List */}
        {selectedUserId && (
          <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-white">
                Restrictions for: {selectedUserId}
              </h2>
              {!activeRestriction && (
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
                >
                  <Plus size={18} />
                  Create Restriction
                </button>
              )}
            </div>

            {isLoading ? (
              <div className="text-center py-8 text-slate-400">Loading...</div>
            ) : error ? (
              <div className="text-center py-8 text-red-400">
                Error: {(error as Error).message}
              </div>
            ) : restrictions.length === 0 ? (
              <div className="text-center py-8 text-slate-400">
                No restrictions found for this user
              </div>
            ) : (
              <div className="space-y-3">
                {restrictions.map((restriction) => (
                  <div
                    key={restriction.restriction_id}
                    className="bg-slate-700/50 border border-slate-600 rounded-lg p-4"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <span
                            className={`px-3 py-1 rounded-full text-xs font-medium ${
                              restriction.status === "ACTIVE"
                                ? "bg-orange-900/50 text-orange-400"
                                : restriction.status === "LIFTED"
                                ? "bg-green-900/50 text-green-400"
                                : "bg-slate-600 text-slate-400"
                            }`}
                          >
                            {restriction.status}
                          </span>
                          <span className="text-sm text-slate-400">
                            {restriction.restriction_type}
                          </span>
                        </div>
                        <p className="text-sm text-slate-300 font-mono mb-1">
                          {restriction.restriction_id}
                        </p>
                        <p className="text-xs text-slate-500">
                          Requested: {new Date(restriction.requested_at).toLocaleString()}
                        </p>
                        {restriction.expires_at && (
                          <p className="text-xs text-slate-500">
                            Expires: {new Date(restriction.expires_at).toLocaleString()}
                          </p>
                        )}
                      </div>
                      {restriction.status === "ACTIVE" && (
                        <CheckCircle className="text-orange-400" size={20} />
                      )}
                      {restriction.status === "LIFTED" && (
                        <Unlock className="text-green-400" size={20} />
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Create Restriction Modal */}
        {showCreateModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Create Processing Restriction
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Restriction Type
                  </label>
                  <select
                    value={restrictionType}
                    onChange={(e) => setRestrictionType(e.target.value)}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  >
                    <option value="FULL">FULL - Block all processing</option>
                    <option value="PARTIAL">PARTIAL - Block specific actions</option>
                    <option value="SPECIFIC_ACTION">SPECIFIC_ACTION - Block specific actions</option>
                  </select>
                </div>

                {(restrictionType === "PARTIAL" ||
                  restrictionType === "SPECIFIC_ACTION") && (
                  <div>
                    <label className="block text-sm font-medium text-slate-300 mb-2">
                      Restricted Actions (comma-separated)
                    </label>
                    <input
                      type="text"
                      value={restrictedActions}
                      onChange={(e) => setRestrictedActions(e.target.value)}
                      placeholder="credit_scoring, automated_decision"
                      className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                    />
                  </div>
                )}

                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="Reason for restriction"
                    rows={3}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Expires At (optional, format: YYYY-MM-DD HH:MM:SS)
                  </label>
                  <input
                    type="text"
                    value={expiresAt}
                    onChange={(e) => setExpiresAt(e.target.value)}
                    placeholder="2025-12-31 23:59:59"
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

        {/* Lift Restriction Modal */}
        {showLiftModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Lift Processing Restriction
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="Reason for lifting restriction"
                    rows={3}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowLiftModal(false)}
                    className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={() => liftMutation.mutate()}
                    disabled={liftMutation.isPending}
                    className="flex-1 px-4 py-2 bg-orange-600 hover:bg-orange-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                  >
                    {liftMutation.isPending ? "Lifting..." : "Lift Restriction"}
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

