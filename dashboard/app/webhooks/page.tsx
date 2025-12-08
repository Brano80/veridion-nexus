"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Webhook, Plus, Trash2, Edit, CheckCircle, XCircle } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchWebhooks() {
  const res = await fetch(`${API_BASE}/webhooks`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch webhooks: ${res.status}`);
  }
  return res.json();
}

async function deleteWebhook(id: string) {
  const res = await fetch(`${API_BASE}/webhooks/${id}`, {
    method: "DELETE",
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to delete webhook: ${res.status}`);
  }
  return res.json();
}

export default function WebhooksPage() {
  const queryClient = useQueryClient();
  const [showAddModal, setShowAddModal] = useState(false);
  const [newWebhook, setNewWebhook] = useState({
    endpoint_url: "",
    event_types: [] as string[],
  });

  const { data: data, isLoading } = useQuery({
    queryKey: ["webhooks"],
    queryFn: fetchWebhooks,
  });

  const webhooks = data?.endpoints || [];

  const deleteMutation = useMutation({
    mutationFn: deleteWebhook,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["webhooks"] });
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

  const activeWebhooks = webhooks.filter((w: any) => w.active);

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">
              Webhook Management
            </h1>
            <p className="text-slate-400">
              Configure real-time compliance event notifications
            </p>
          </div>
          <button
            onClick={() => setShowAddModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-emerald-900/50 hover:bg-emerald-800/80 text-emerald-400 border border-emerald-800 rounded-lg transition-colors"
          >
            <Plus size={18} />
            Add Webhook
          </button>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-emerald-900/20 border border-emerald-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Webhook className="text-emerald-400" size={24} />
              <div className="text-3xl font-bold text-emerald-400">
                {activeWebhooks.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Active Webhooks</div>
          </div>
          <div className="p-6 bg-blue-900/20 border border-blue-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle className="text-blue-400" size={24} />
              <div className="text-3xl font-bold text-blue-400">
                {webhooks.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Total Webhooks</div>
          </div>
        </div>

        {/* Webhooks List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Webhook Endpoints</h2>
          <div className="space-y-3">
            {webhooks.map((webhook: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          webhook.active
                            ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                            : "bg-slate-800 text-slate-500 border border-slate-700"
                        }`}
                      >
                        {webhook.active ? "Active" : "Inactive"}
                      </span>
                      <span className="text-sm font-medium text-white">
                        {webhook.endpoint_url}
                      </span>
                    </div>
                    <div className="text-xs text-slate-500 mb-2">
                      ID: {webhook.id} • Created: {webhook.created_at}
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {webhook.event_types?.map((type: string, j: number) => (
                        <span
                          key={j}
                          className="px-2 py-1 rounded text-xs bg-blue-900/30 text-blue-400 border border-blue-800"
                        >
                          {type}
                        </span>
                      ))}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => deleteMutation.mutate(webhook.id)}
                      className="p-2 text-slate-500 hover:text-red-400 hover:bg-red-900/20 rounded transition-colors"
                      title="Delete webhook"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {webhooks.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              No webhooks configured. Add one to get started.
            </div>
          )}
        </div>

        {/* Add Modal */}
        {showAddModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
            <div className="bg-slate-900 border border-slate-800 rounded-lg p-6 max-w-2xl w-full">
              <h3 className="text-xl font-bold text-white mb-4">
                Add Webhook Endpoint
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="text-sm text-slate-400 mb-2 block">
                    Endpoint URL
                  </label>
                  <input
                    type="url"
                    value={newWebhook.endpoint_url}
                    onChange={(e) =>
                      setNewWebhook({
                        ...newWebhook,
                        endpoint_url: e.target.value,
                      })
                    }
                    className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
                    placeholder="https://example.com/webhooks"
                  />
                </div>
                <div>
                  <label className="text-sm text-slate-400 mb-2 block">
                    Event Types (comma-separated)
                  </label>
                  <input
                    type="text"
                    value={newWebhook.event_types.join(", ")}
                    onChange={(e) =>
                      setNewWebhook({
                        ...newWebhook,
                        event_types: e.target.value
                          .split(",")
                          .map((s) => s.trim())
                          .filter(Boolean),
                      })
                    }
                    className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
                    placeholder="compliance.action, data_breach.detected"
                  />
                </div>
              </div>
              <div className="flex gap-3 justify-end mt-6">
                <button
                  onClick={() => {
                    setShowAddModal(false);
                    setNewWebhook({ endpoint_url: "", event_types: [] });
                  }}
                  className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={async () => {
                    try {
                      const res = await fetch(`${API_BASE}/webhooks`, {
                        method: "POST",
                        headers: getAuthHeaders(),
                        body: JSON.stringify(newWebhook),
                      });
                      if (res.ok) {
                        queryClient.invalidateQueries({ queryKey: ["webhooks"] });
                        setShowAddModal(false);
                        setNewWebhook({ endpoint_url: "", event_types: [] });
                      } else {
                        if (res.status === 401) {
                          alert("Unauthorized - Please login");
                        } else {
                          alert("Failed to create webhook");
                        }
                      }
                    } catch (error) {
                      alert("Failed to create webhook");
                    }
                  }}
                  className="px-4 py-2 bg-emerald-900/50 hover:bg-emerald-800/80 text-emerald-400 border border-emerald-800 rounded-lg transition-colors"
                >
                  Add Webhook
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

