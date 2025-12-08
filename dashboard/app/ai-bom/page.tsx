"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Package, Download, FileJson } from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchAIBOM(systemId: string) {
  const res = await fetch(`${API_BASE}/ai_bom/${systemId}`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) {
    if (res.status === 401) {
      throw new Error("Unauthorized - Please login");
    }
    throw new Error(`Failed to fetch AI-BOM: ${res.status}`);
  }
  return res.json();
}

export default function AIBOMPage() {
  const [systemId, setSystemId] = useState("AI-SYSTEM-001");
  const { data: bom, isLoading, refetch } = useQuery({
    queryKey: ["ai-bom", systemId],
    queryFn: () => fetchAIBOM(systemId),
    enabled: false,
  });

  const handleExport = () => {
    refetch();
  };

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">AI-BOM Viewer</h1>
          <p className="text-slate-400">
            CycloneDX AI-BOM export and visualization
          </p>
        </div>

        {/* Export Form */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">
            Export AI-BOM
          </h2>
          <div className="flex gap-4 items-end">
            <div className="flex-1">
              <label className="text-sm text-slate-400 mb-2 block">
                System ID
              </label>
              <input
                type="text"
                value={systemId}
                onChange={(e) => setSystemId(e.target.value)}
                className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
                placeholder="AI-SYSTEM-001"
              />
            </div>
            <button
              onClick={handleExport}
              className="flex items-center gap-2 px-4 py-2 bg-emerald-900/50 hover:bg-emerald-800/80 text-emerald-400 border border-emerald-800 rounded-lg transition-colors"
            >
              <Download size={18} />
              Export BOM
            </button>
          </div>
        </div>

        {/* BOM Display */}
        {bom && (
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-white">BOM Details</h2>
              <button
                onClick={() => {
                  const blob = new Blob([JSON.stringify(bom, null, 2)], {
                    type: "application/json",
                  });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement("a");
                  a.href = url;
                  a.download = `ai-bom-${systemId}.json`;
                  a.click();
                }}
                className="flex items-center gap-2 px-4 py-2 bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 rounded-lg transition-colors"
              >
                <FileJson size={18} />
                Download JSON
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <span className="text-sm text-slate-400">BOM Format: </span>
                <span className="text-white">{bom.bomFormat}</span>
              </div>
              <div>
                <span className="text-sm text-slate-400">Spec Version: </span>
                <span className="text-white">{bom.specVersion}</span>
              </div>
              <div>
                <span className="text-sm text-slate-400">Components: </span>
                <span className="text-white">{bom.components?.length || 0}</span>
              </div>
              <div className="mt-4">
                <h3 className="text-lg font-bold text-white mb-2">Components</h3>
                <div className="space-y-2">
                  {bom.components?.map((comp: any, i: number) => (
                    <div
                      key={i}
                      className="p-3 bg-slate-800/50 rounded border border-slate-700"
                    >
                      <div className="text-sm font-medium text-white">
                        {comp.name}
                      </div>
                      <div className="text-xs text-slate-500">
                        Type: {comp.component_type} • Version: {comp.version || "N/A"}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}

        {!bom && !isLoading && (
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="text-center py-12 text-slate-500">
              <Package size={48} className="mx-auto mb-4 opacity-50" />
              <p>Enter a System ID and click Export to view AI-BOM</p>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

