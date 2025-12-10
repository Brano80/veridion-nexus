"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { Package, Plus, Edit, Tag, Building2, MapPin, AlertTriangle, Users } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

interface Asset {
  id: string;
  asset_id: string;
  asset_name: string;
  asset_type: string;
  business_function: string;
  department: string | null;
  owner: string | null;
  location: string | null;
  risk_profile: string;
  tags: string[] | null;
  agent_ids: string[];
  created_at: string;
}

interface BusinessFunction {
  id: string;
  function_code: string;
  function_name: string;
  description: string | null;
  default_risk_level: string;
  compliance_requirements: string[] | null;
}

async function fetchAssets() {
  const res = await fetch(`${API_BASE}/assets`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch assets: ${res.status}`);
  }
  const data = await res.json();
  return data.assets || [];
}

async function fetchBusinessFunctions() {
  const res = await fetch(`${API_BASE}/business-functions`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch business functions: ${res.status}`);
  }
  return res.json() as Promise<BusinessFunction[]>;
}

async function createOrUpdateAsset(asset: any) {
  const res = await fetch(`${API_BASE}/assets`, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify(asset),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to save asset: ${res.status}`);
  }
  return res.json();
}

export default function AssetsPage() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingAsset, setEditingAsset] = useState<Asset | null>(null);
  const [formData, setFormData] = useState({
    asset_id: "",
    asset_name: "",
    asset_type: "AI_AGENT",
    business_function: "",
    department: "",
    owner: "",
    location: "",
    risk_profile: "MEDIUM",
    tags: "",
    agent_ids: "",
  });

  const queryClient = useQueryClient();

  const { data: assets = [], isLoading, refetch } = useQuery({
    queryKey: ["assets"],
    queryFn: fetchAssets,
  });

  const { data: businessFunctions = [] } = useQuery({
    queryKey: ["business-functions"],
    queryFn: fetchBusinessFunctions,
  });

  const createMutation = useMutation({
    mutationFn: createOrUpdateAsset,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["assets"] });
      setShowCreateModal(false);
      setEditingAsset(null);
      setFormData({
        asset_id: "",
        asset_name: "",
        asset_type: "AI_AGENT",
        business_function: "",
        department: "",
        owner: "",
        location: "",
        risk_profile: "MEDIUM",
        tags: "",
        agent_ids: "",
      });
    },
    onError: (error: Error) => {
      alert(`Failed to save asset: ${error.message}`);
    },
  });

  const handleEdit = (asset: Asset) => {
    setEditingAsset(asset);
    setFormData({
      asset_id: asset.asset_id,
      asset_name: asset.asset_name,
      asset_type: asset.asset_type,
      business_function: asset.business_function,
      department: asset.department || "",
      owner: asset.owner || "",
      location: asset.location || "",
      risk_profile: asset.risk_profile,
      tags: asset.tags?.join(", ") || "",
      agent_ids: asset.agent_ids.join(", ") || "",
    });
    setShowCreateModal(true);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate({
      asset_id: formData.asset_id,
      asset_name: formData.asset_name,
      asset_type: formData.asset_type,
      business_function: formData.business_function,
      department: formData.department || null,
      owner: formData.owner || null,
      location: formData.location || null,
      risk_profile: formData.risk_profile,
      tags: formData.tags ? formData.tags.split(",").map((t) => t.trim()) : null,
      agent_ids: formData.agent_ids ? formData.agent_ids.split(",").map((id) => id.trim()) : null,
    });
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case "CRITICAL":
        return "text-red-400 bg-red-900/20 border-red-800";
      case "HIGH":
        return "text-orange-400 bg-orange-900/20 border-orange-800";
      case "MEDIUM":
        return "text-yellow-400 bg-yellow-900/20 border-yellow-800";
      case "LOW":
        return "text-green-400 bg-green-900/20 border-green-800";
      default:
        return "text-slate-400 bg-slate-900/20 border-slate-800";
    }
  };

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading assets...</div>
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
            <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-2">
              <Package size={28} />
              Asset Registry
            </h1>
            <p className="text-slate-400">
              Context-aware asset management with business function mapping
            </p>
          </div>
          <button
            onClick={() => {
              setEditingAsset(null);
              setFormData({
                asset_id: "",
                asset_name: "",
                asset_type: "AI_AGENT",
                business_function: "",
                department: "",
                owner: "",
                location: "",
                risk_profile: "MEDIUM",
                tags: "",
                agent_ids: "",
              });
              setShowCreateModal(true);
            }}
            className="flex items-center gap-2 px-4 py-2 bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 rounded-lg transition-colors"
          >
            <Plus size={18} />
            Register Asset
          </button>
        </div>

        {/* Assets Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {assets.map((asset: Asset) => (
            <div
              key={asset.id}
              className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 hover:border-slate-600 transition-colors"
            >
              <div className="flex items-start justify-between mb-4">
                <div>
                  <h3 className="text-lg font-semibold text-white mb-1">{asset.asset_name}</h3>
                  <p className="text-sm text-slate-400 font-mono">{asset.asset_id}</p>
                </div>
                <button
                  onClick={() => handleEdit(asset)}
                  className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
                >
                  <Edit size={16} className="text-slate-400" />
                </button>
              </div>

              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm">
                  <Tag size={14} className="text-slate-500" />
                  <span className="text-slate-300">{asset.business_function}</span>
                </div>
                {asset.department && (
                  <div className="flex items-center gap-2 text-sm">
                    <Building2 size={14} className="text-slate-500" />
                    <span className="text-slate-300">{asset.department}</span>
                  </div>
                )}
                {asset.location && (
                  <div className="flex items-center gap-2 text-sm">
                    <MapPin size={14} className="text-slate-500" />
                    <span className="text-slate-300">{asset.location}</span>
                  </div>
                )}
                <div className="flex items-center gap-2">
                  <AlertTriangle size={14} className="text-slate-500" />
                  <span className={`px-2 py-1 rounded text-xs border ${getRiskColor(asset.risk_profile)}`}>
                    {asset.risk_profile}
                  </span>
                </div>
                {asset.agent_ids.length > 0 && (
                  <div className="flex items-center gap-2 text-sm">
                    <Users size={14} className="text-slate-500" />
                    <span className="text-slate-300">{asset.agent_ids.length} agent(s)</span>
                  </div>
                )}
                {asset.tags && asset.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1 mt-2">
                    {asset.tags.map((tag, idx) => (
                      <span
                        key={idx}
                        className="px-2 py-1 bg-slate-700/50 text-slate-300 text-xs rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {assets.length === 0 && (
          <div className="text-center py-12 text-slate-400">
            No assets registered. Click "Register Asset" to get started.
          </div>
        )}

        {/* Create/Edit Modal */}
        {showCreateModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
              <h2 className="text-2xl font-bold text-white mb-4">
                {editingAsset ? "Edit Asset" : "Register New Asset"}
              </h2>
              <form onSubmit={handleSubmit} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Asset ID *</label>
                    <input
                      type="text"
                      required
                      value={formData.asset_id}
                      onChange={(e) => setFormData({ ...formData, asset_id: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                      placeholder="agent-credit-scoring-001"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Asset Name *</label>
                    <input
                      type="text"
                      required
                      value={formData.asset_name}
                      onChange={(e) => setFormData({ ...formData, asset_name: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                      placeholder="Credit Scoring AI Agent"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Asset Type *</label>
                    <select
                      required
                      value={formData.asset_type}
                      onChange={(e) => setFormData({ ...formData, asset_type: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                    >
                      <option value="AI_AGENT">AI Agent</option>
                      <option value="API_SERVICE">API Service</option>
                      <option value="DATA_PROCESSOR">Data Processor</option>
                      <option value="ML_MODEL">ML Model</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Business Function *</label>
                    <select
                      required
                      value={formData.business_function}
                      onChange={(e) => setFormData({ ...formData, business_function: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                    >
                      <option value="">Select...</option>
                      {businessFunctions.map((bf) => (
                        <option key={bf.function_code} value={bf.function_code}>
                          {bf.function_name}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Department</label>
                    <input
                      type="text"
                      value={formData.department}
                      onChange={(e) => setFormData({ ...formData, department: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                      placeholder="RISK_MANAGEMENT"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Owner</label>
                    <input
                      type="text"
                      value={formData.owner}
                      onChange={(e) => setFormData({ ...formData, owner: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                      placeholder="john.doe@company.com"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Location</label>
                    <input
                      type="text"
                      value={formData.location}
                      onChange={(e) => setFormData({ ...formData, location: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                      placeholder="EU"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-slate-400 mb-2">Risk Profile</label>
                    <select
                      value={formData.risk_profile}
                      onChange={(e) => setFormData({ ...formData, risk_profile: e.target.value })}
                      className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                    >
                      <option value="LOW">Low</option>
                      <option value="MEDIUM">Medium</option>
                      <option value="HIGH">High</option>
                      <option value="CRITICAL">Critical</option>
                    </select>
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">Tags (comma-separated)</label>
                  <input
                    type="text"
                    value={formData.tags}
                    onChange={(e) => setFormData({ ...formData, tags: e.target.value })}
                    className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                    placeholder="production, critical, eu-only"
                  />
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">Agent IDs (comma-separated)</label>
                  <input
                    type="text"
                    value={formData.agent_ids}
                    onChange={(e) => setFormData({ ...formData, agent_ids: e.target.value })}
                    className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
                    placeholder="agent-1, agent-2"
                  />
                </div>

                <div className="flex gap-2 justify-end">
                  <button
                    type="button"
                    onClick={() => {
                      setShowCreateModal(false);
                      setEditingAsset(null);
                    }}
                    className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={createMutation.isPending}
                    className="px-4 py-2 bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 rounded-lg transition-colors disabled:opacity-50"
                  >
                    {createMutation.isPending ? "Saving..." : editingAsset ? "Update" : "Create"}
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

