"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Bot, Search, AlertCircle, CheckCircle, XCircle, Clock, FileText } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface AutomatedDecision {
  decision_id: string;
  user_id: string;
  seal_id: string;
  action_type: string;
  decision_outcome: string;
  decision_reasoning?: string;
  legal_effect?: string;
  significant_impact: boolean;
  status: string;
  decision_timestamp: string;
  human_review_required: boolean;
}

interface AutomatedDecisionsResponse {
  user_id: string;
  decisions: AutomatedDecision[];
}

async function fetchDecisions(userId: string): Promise<AutomatedDecisionsResponse> {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/automated_decisions`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    throw new Error(`Failed to fetch decisions: ${res.status}`);
  }
  return res.json();
}

async function requestReview(userId: string, decisionId: string, sealId?: string, reason?: string) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/request_review`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      user_id: userId,
      decision_id: decisionId || undefined,
      seal_id: sealId || undefined,
      reason: reason || undefined,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to request review");
  }
  return res.json();
}

async function appealDecision(userId: string, decisionId: string, appealReason: string) {
  const res = await fetch(`${API_BASE}/data_subject/${userId}/appeal_decision`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      user_id: userId,
      decision_id: decisionId,
      appeal_reason: appealReason,
    }),
  });
  if (!res.ok) {
    if (res.status === 401) {
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || "Failed to appeal decision");
  }
  return res.json();
}

export default function AutomatedDecisionsPage() {
  const [searchUserId, setSearchUserId] = useState("");
  const [selectedUserId, setSelectedUserId] = useState("");
  const [showReviewModal, setShowReviewModal] = useState(false);
  const [showAppealModal, setShowAppealModal] = useState(false);
  const [selectedDecision, setSelectedDecision] = useState<AutomatedDecision | null>(null);
  const [reviewReason, setReviewReason] = useState("");
  const [appealReason, setAppealReason] = useState("");
  const queryClient = useQueryClient();

  const { data: decisionsData, isLoading, error } = useQuery({
    queryKey: ["automated-decisions", selectedUserId],
    queryFn: () => fetchDecisions(selectedUserId),
    enabled: !!selectedUserId,
  });

  const reviewMutation = useMutation({
    mutationFn: (data: { decisionId: string; sealId?: string; reason?: string }) =>
      requestReview(selectedUserId, data.decisionId, data.sealId, data.reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["automated-decisions", selectedUserId] });
      setShowReviewModal(false);
      setReviewReason("");
      setSelectedDecision(null);
      alert("Human review requested successfully");
    },
    onError: (err: any) => {
      alert(`Failed to request review: ${err.message}`);
    },
  });

  const appealMutation = useMutation({
    mutationFn: (data: { decisionId: string; reason: string }) =>
      appealDecision(selectedUserId, data.decisionId, data.reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["automated-decisions", selectedUserId] });
      setShowAppealModal(false);
      setAppealReason("");
      setSelectedDecision(null);
      alert("Appeal requested successfully");
    },
    onError: (err: any) => {
      alert(`Failed to appeal decision: ${err.message}`);
    },
  });

  const handleSearch = () => {
    if (!searchUserId.trim()) {
      alert("Please enter a user ID");
      return;
    }
    setSelectedUserId(searchUserId.trim());
  };

  const handleRequestReview = (decision: AutomatedDecision) => {
    setSelectedDecision(decision);
    setShowReviewModal(true);
  };

  const handleAppeal = (decision: AutomatedDecision) => {
    setSelectedDecision(decision);
    setShowAppealModal(true);
  };

  const submitReview = () => {
    if (!selectedDecision) return;
    reviewMutation.mutate({
      decisionId: selectedDecision.decision_id,
      sealId: selectedDecision.seal_id,
      reason: reviewReason || undefined,
    });
  };

  const submitAppeal = () => {
    if (!selectedDecision || !appealReason.trim()) {
      alert("Please provide an appeal reason");
      return;
    }
    appealMutation.mutate({
      decisionId: selectedDecision.decision_id,
      reason: appealReason,
    });
  };

  const decisions = decisionsData?.decisions || [];
  const pendingDecisions = decisions.filter((d) => d.status === "PENDING_REVIEW" || d.status === "UNDER_REVIEW");

  const getStatusColor = (status: string) => {
    switch (status) {
      case "PENDING_REVIEW":
      case "UNDER_REVIEW":
        return "bg-yellow-900/50 text-yellow-400";
      case "REVIEWED":
        return "bg-blue-900/50 text-blue-400";
      case "APPEALED":
        return "bg-orange-900/50 text-orange-400";
      case "OVERRIDDEN":
        return "bg-green-900/50 text-green-400";
      default:
        return "bg-slate-600 text-slate-400";
    }
  };

  const getOutcomeColor = (outcome: string) => {
    switch (outcome) {
      case "APPROVED":
        return "text-green-400";
      case "REJECTED":
        return "text-red-400";
      case "CONDITIONAL":
        return "text-yellow-400";
      default:
        return "text-slate-400";
    }
  };

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
            <Bot className="text-purple-400" size={32} />
            Automated Decisions
          </h1>
          <p className="text-slate-400">
            GDPR Article 22 - Right to Automated Decision-Making
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
                  placeholder="Enter user ID to view automated decisions"
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

        {/* Pending Reviews Alert */}
        {pendingDecisions.length > 0 && (
          <div className="bg-yellow-900/30 border border-yellow-700 rounded-lg p-4 flex items-center gap-3">
            <AlertCircle className="text-yellow-400" size={24} />
            <div className="flex-1">
              <p className="text-yellow-400 font-medium">
                {pendingDecisions.length} Decision(s) Pending Review
              </p>
              <p className="text-slate-400 text-sm">
                These automated decisions require human review
              </p>
            </div>
          </div>
        )}

        {/* Decisions List */}
        {selectedUserId && (
          <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-white">
                Automated Decisions for: {selectedUserId}
              </h2>
            </div>

            {isLoading ? (
              <div className="text-center py-8 text-slate-400">Loading...</div>
            ) : error ? (
              <div className="text-center py-8 text-red-400">
                Error: {(error as Error).message}
              </div>
            ) : decisions.length === 0 ? (
              <div className="text-center py-8 text-slate-400">
                No automated decisions found for this user
              </div>
            ) : (
              <div className="space-y-4">
                {decisions.map((decision) => (
                  <div
                    key={decision.decision_id}
                    className="bg-slate-700/50 border border-slate-600 rounded-lg p-4"
                  >
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(decision.status)}`}>
                            {decision.status}
                          </span>
                          <span className={`text-sm font-medium ${getOutcomeColor(decision.decision_outcome)}`}>
                            {decision.decision_outcome}
                          </span>
                          <span className="text-sm text-slate-400">
                            {decision.action_type}
                          </span>
                        </div>
                        <p className="text-sm text-slate-300 font-mono mb-1">
                          {decision.decision_id}
                        </p>
                        <p className="text-xs text-slate-500 mb-2">
                          Decision made: {new Date(decision.decision_timestamp).toLocaleString()}
                        </p>
                        {decision.legal_effect && (
                          <div className="bg-red-900/20 border border-red-700/50 rounded p-2 mb-2">
                            <p className="text-xs text-red-400">
                              <strong>Legal Effect:</strong> {decision.legal_effect}
                            </p>
                          </div>
                        )}
                        {decision.decision_reasoning && (
                          <div className="bg-slate-600/50 rounded p-2 mb-2">
                            <p className="text-xs text-slate-300">
                              <strong>Reasoning:</strong> {decision.decision_reasoning}
                            </p>
                          </div>
                        )}
                        {decision.significant_impact && (
                          <p className="text-xs text-orange-400 mb-2">
                            ⚠️ This decision significantly affects the individual
                          </p>
                        )}
                      </div>
                      <div className="flex flex-col gap-2">
                        {decision.human_review_required && (
                          <Bot className="text-purple-400" size={20} />
                        )}
                        {(decision.status === "PENDING_REVIEW" || decision.status === "UNDER_REVIEW") && (
                          <Clock className="text-yellow-400" size={20} />
                        )}
                        {decision.status === "REVIEWED" && (
                          <CheckCircle className="text-green-400" size={20} />
                        )}
                        {decision.status === "APPEALED" && (
                          <FileText className="text-orange-400" size={20} />
                        )}
                      </div>
                    </div>
                    <div className="flex gap-2 mt-3">
                      {(decision.status === "PENDING_REVIEW" || decision.status === "UNDER_REVIEW") && (
                        <button
                          onClick={() => handleRequestReview(decision)}
                          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
                        >
                          <FileText size={16} />
                          Request Review
                        </button>
                      )}
                      {decision.status === "REVIEWED" && (
                        <button
                          onClick={() => handleAppeal(decision)}
                          className="px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
                        >
                          <AlertCircle size={16} />
                          Appeal Decision
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Request Review Modal */}
        {showReviewModal && selectedDecision && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Request Human Review
              </h3>
              <div className="space-y-4">
                <div className="bg-blue-900/20 border border-blue-700/50 rounded-lg p-3 mb-4">
                  <p className="text-xs text-blue-400">
                    <strong>GDPR Article 22:</strong> You have the right to request human review of automated decisions.
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Decision ID
                  </label>
                  <input
                    type="text"
                    value={selectedDecision.decision_id}
                    disabled
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-slate-400"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reviewReason}
                    onChange={(e) => setReviewReason(e.target.value)}
                    placeholder="Reason for requesting review"
                    rows={3}
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowReviewModal(false)}
                    className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={submitReview}
                    disabled={reviewMutation.isPending}
                    className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                  >
                    {reviewMutation.isPending ? "Requesting..." : "Request Review"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Appeal Decision Modal */}
        {showAppealModal && selectedDecision && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-md">
              <h3 className="text-xl font-bold text-white mb-4">
                Appeal Automated Decision
              </h3>
              <div className="space-y-4">
                <div className="bg-orange-900/20 border border-orange-700/50 rounded-lg p-3 mb-4">
                  <p className="text-xs text-orange-400">
                    <strong>GDPR Article 22:</strong> You have the right to appeal automated decisions that produce legal effects.
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Decision ID
                  </label>
                  <input
                    type="text"
                    value={selectedDecision.decision_id}
                    disabled
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-slate-400"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Appeal Reason <span className="text-red-400">*</span>
                  </label>
                  <textarea
                    value={appealReason}
                    onChange={(e) => setAppealReason(e.target.value)}
                    placeholder="Explain why you believe the decision was incorrect..."
                    rows={4}
                    required
                    className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-emerald-500"
                  />
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowAppealModal(false)}
                    className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={submitAppeal}
                    disabled={appealMutation.isPending || !appealReason.trim()}
                    className="flex-1 px-4 py-2 bg-orange-600 hover:bg-orange-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                  >
                    {appealMutation.isPending ? "Submitting..." : "Submit Appeal"}
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

