"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Shield, 
  AlertTriangle, 
  Building2, 
  Activity, 
  CheckCircle, 
  XCircle, 
  Plus, 
  Download,
  Clock,
  TrendingUp,
  FileText,
  Server
} from "lucide-react";
import { useState } from "react";
import { getAuthHeaders } from "../utils/auth";
import { format } from "date-fns";

const API_BASE = "http://127.0.0.1:8080/api/v1";

// Types
interface DORALiteComplianceStatus {
  compliance_score: number;
  article9_compliant: boolean;
  article10_compliant: boolean;
  article11_compliant: boolean;
  vendor_count: number;
  incident_count: number;
  sla_count: number;
  recommendations: string[];
}

interface DORALiteIncident {
  id: string;
  incident_type: string;
  description: string;
  detected_at: string;
  resolved_at: string | null;
  severity: string;
  status: string;
  impact_description: string | null;
  mitigation_steps: string | null;
  reported_to_authority: boolean;
  reported_at: string | null;
  created_at: string;
  updated_at: string;
}

interface DORALiteVendor {
  id: string;
  vendor_name: string;
  vendor_type: string;
  service_description: string | null;
  country_code: string | null;
  contact_email: string | null;
  sla_uptime_percentage: number | null;
  last_reviewed_at: string | null;
  risk_level: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

interface DORALiteSLA {
  id: string;
  service_name: string;
  service_type: string;
  sla_target_uptime: number;
  actual_uptime: number | null;
  monitoring_period_start: string;
  monitoring_period_end: string;
  downtime_minutes: number;
  incidents_count: number;
  sla_met: boolean;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

// API Functions
async function fetchComplianceStatus(): Promise<DORALiteComplianceStatus> {
  const res = await fetch(`${API_BASE}/dora-lite/compliance-status`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

async function fetchIncidents(): Promise<{ incidents: DORALiteIncident[]; total: number; open_incidents: number; resolved_incidents: number }> {
  const res = await fetch(`${API_BASE}/dora-lite/incidents`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

async function fetchVendors(): Promise<DORALiteVendor[]> {
  const res = await fetch(`${API_BASE}/dora-lite/vendors`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

async function fetchSLAs(): Promise<DORALiteSLA[]> {
  const res = await fetch(`${API_BASE}/dora-lite/sla-monitoring`, {
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
  return res.json();
}

function getSeverityColor(severity: string) {
  switch (severity.toUpperCase()) {
    case "CRITICAL": return "text-red-400 bg-red-500/10 border-red-500/20";
    case "HIGH": return "text-orange-400 bg-orange-500/10 border-orange-500/20";
    case "MEDIUM": return "text-yellow-400 bg-yellow-500/10 border-yellow-500/20";
    case "LOW": return "text-blue-400 bg-blue-500/10 border-blue-500/20";
    default: return "text-slate-400 bg-slate-500/10 border-slate-500/20";
  }
}

function getStatusColor(status: string) {
  switch (status.toUpperCase()) {
    case "OPEN": return "text-red-400";
    case "IN_PROGRESS": return "text-yellow-400";
    case "RESOLVED": return "text-emerald-400";
    case "CLOSED": return "text-slate-400";
    default: return "text-slate-400";
  }
}

function getRiskLevelColor(risk: string) {
  switch (risk.toUpperCase()) {
    case "HIGH": return "text-red-400 bg-red-500/10";
    case "MEDIUM": return "text-yellow-400 bg-yellow-500/10";
    case "LOW": return "text-emerald-400 bg-emerald-500/10";
    default: return "text-slate-400 bg-slate-500/10";
  }
}

export default function DORALitePage() {
  const [activeTab, setActiveTab] = useState<"overview" | "incidents" | "vendors" | "sla">("overview");

  const { data: complianceStatus, isLoading: statusLoading } = useQuery({
    queryKey: ["dora-lite-compliance"],
    queryFn: fetchComplianceStatus,
    refetchInterval: 60000,
  });

  const { data: incidentsData, isLoading: incidentsLoading } = useQuery({
    queryKey: ["dora-lite-incidents"],
    queryFn: fetchIncidents,
    refetchInterval: 30000,
  });

  const { data: vendors, isLoading: vendorsLoading } = useQuery({
    queryKey: ["dora-lite-vendors"],
    queryFn: fetchVendors,
    refetchInterval: 60000,
  });

  const { data: slas, isLoading: slasLoading } = useQuery({
    queryKey: ["dora-lite-slas"],
    queryFn: fetchSLAs,
    refetchInterval: 60000,
  });

  if (statusLoading) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="animate-pulse space-y-4">
            <div className="h-8 bg-slate-800 rounded w-1/3"></div>
            <div className="h-64 bg-slate-800 rounded"></div>
          </div>
        </div>
      </DashboardLayout>
    );
  }

  if (!complianceStatus) {
    return (
      <DashboardLayout>
        <div className="p-8">
          <div className="text-red-400">Failed to load DORA Lite data</div>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">DORA Lite Compliance</h1>
            <p className="text-slate-400">
              Simplified DORA compliance for Startups/SMEs (Principle of Proportionality)
            </p>
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b border-slate-700">
          <nav className="flex space-x-8">
            {[
              { id: "overview", label: "Overview", icon: Shield },
              { id: "incidents", label: "Incidents", icon: AlertTriangle },
              { id: "vendors", label: "Vendors", icon: Building2 },
              { id: "sla", label: "SLA Monitoring", icon: Activity },
            ].map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`flex items-center space-x-2 py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === tab.id
                      ? "border-blue-500 text-blue-400"
                      : "border-transparent text-slate-400 hover:text-slate-300 hover:border-slate-600"
                  }`}
                >
                  <Icon size={18} />
                  <span>{tab.label}</span>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Tab Content */}
        {activeTab === "overview" && (
          <div className="space-y-6">
            {/* Compliance Score Card */}
            <div className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-white">Compliance Score</h2>
                <div className={`text-4xl font-bold ${
                  complianceStatus.compliance_score >= 80 ? "text-emerald-400" :
                  complianceStatus.compliance_score >= 60 ? "text-yellow-400" :
                  "text-red-400"
                }`}>
                  {complianceStatus.compliance_score.toFixed(1)}%
                </div>
              </div>
              <div className="w-full bg-slate-700 rounded-full h-3 mb-6">
                <div
                  className={`h-3 rounded-full ${
                    complianceStatus.compliance_score >= 80 ? "bg-emerald-500" :
                    complianceStatus.compliance_score >= 60 ? "bg-yellow-500" :
                    "bg-red-500"
                  }`}
                  style={{ width: `${complianceStatus.compliance_score}%` }}
                />
              </div>

              {/* Article Compliance Status */}
              <div className="grid grid-cols-3 gap-4">
                <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-slate-400 text-sm">Article 9</span>
                    {complianceStatus.article9_compliant ? (
                      <CheckCircle className="text-emerald-400" size={20} />
                    ) : (
                      <XCircle className="text-red-400" size={20} />
                    )}
                  </div>
                  <div className="text-white font-semibold">Vendor List</div>
                  <div className="text-slate-400 text-sm mt-1">{complianceStatus.vendor_count} vendors</div>
                </div>

                <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-slate-400 text-sm">Article 10</span>
                    {complianceStatus.article10_compliant ? (
                      <CheckCircle className="text-emerald-400" size={20} />
                    ) : (
                      <XCircle className="text-red-400" size={20} />
                    )}
                  </div>
                  <div className="text-white font-semibold">Incident Log</div>
                  <div className="text-slate-400 text-sm mt-1">{complianceStatus.incident_count} incidents</div>
                </div>

                <div className="bg-slate-900/50 rounded-lg p-4 border border-slate-700">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-slate-400 text-sm">Article 11</span>
                    {complianceStatus.article11_compliant ? (
                      <CheckCircle className="text-emerald-400" size={20} />
                    ) : (
                      <XCircle className="text-red-400" size={20} />
                    )}
                  </div>
                  <div className="text-white font-semibold">SLA Monitoring</div>
                  <div className="text-slate-400 text-sm mt-1">{complianceStatus.sla_count} SLAs</div>
                </div>
              </div>

              {/* Recommendations */}
              {complianceStatus.recommendations.length > 0 && (
                <div className="mt-6">
                  <h3 className="text-white font-semibold mb-3">Recommendations</h3>
                  <ul className="space-y-2">
                    {complianceStatus.recommendations.map((rec, idx) => (
                      <li key={idx} className="text-slate-300 text-sm flex items-start">
                        <span className="text-blue-400 mr-2">•</span>
                        {rec}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === "incidents" && (
          <div className="space-y-4">
            {incidentsLoading ? (
              <div className="text-slate-400">Loading incidents...</div>
            ) : incidentsData && incidentsData.incidents.length > 0 ? (
              <div className="space-y-4">
                {incidentsData.incidents.map((incident) => (
                  <div key={incident.id} className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <div className="flex items-center space-x-3 mb-2">
                          <h3 className="text-white font-semibold">{incident.incident_type}</h3>
                          <span className={`px-2 py-1 rounded text-xs font-medium ${getSeverityColor(incident.severity)}`}>
                            {incident.severity}
                          </span>
                          <span className={`text-sm ${getStatusColor(incident.status)}`}>
                            {incident.status}
                          </span>
                        </div>
                        <p className="text-slate-300">{incident.description}</p>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <span className="text-slate-400">Detected:</span>
                        <span className="text-white ml-2">{format(new Date(incident.detected_at), "PPp")}</span>
                      </div>
                      {incident.resolved_at && (
                        <div>
                          <span className="text-slate-400">Resolved:</span>
                          <span className="text-white ml-2">{format(new Date(incident.resolved_at), "PPp")}</span>
                        </div>
                      )}
                      {incident.impact_description && (
                        <div className="col-span-2">
                          <span className="text-slate-400">Impact:</span>
                          <span className="text-white ml-2">{incident.impact_description}</span>
                        </div>
                      )}
                      {incident.mitigation_steps && (
                        <div className="col-span-2">
                          <span className="text-slate-400">Mitigation:</span>
                          <span className="text-white ml-2">{incident.mitigation_steps}</span>
                        </div>
                      )}
                      {incident.reported_to_authority && (
                        <div className="col-span-2">
                          <span className="text-slate-400">Reported to Authority:</span>
                          <span className="text-emerald-400 ml-2">Yes</span>
                          {incident.reported_at && (
                            <span className="text-slate-400 ml-2">
                              ({format(new Date(incident.reported_at), "PPp")})
                            </span>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="bg-slate-800/50 rounded-lg p-8 border border-slate-700 text-center">
                <AlertTriangle className="mx-auto text-slate-500 mb-4" size={48} />
                <p className="text-slate-400">No incidents recorded</p>
              </div>
            )}
          </div>
        )}

        {activeTab === "vendors" && (
          <div className="space-y-4">
            {vendorsLoading ? (
              <div className="text-slate-400">Loading vendors...</div>
            ) : vendors && vendors.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {vendors.map((vendor) => (
                  <div key={vendor.id} className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <h3 className="text-white font-semibold mb-1">{vendor.vendor_name}</h3>
                        <span className="text-slate-400 text-sm">{vendor.vendor_type}</span>
                      </div>
                      <span className={`px-2 py-1 rounded text-xs font-medium ${getRiskLevelColor(vendor.risk_level)}`}>
                        {vendor.risk_level}
                      </span>
                    </div>
                    {vendor.service_description && (
                      <p className="text-slate-300 text-sm mb-4">{vendor.service_description}</p>
                    )}
                    <div className="space-y-2 text-sm">
                      {vendor.country_code && (
                        <div>
                          <span className="text-slate-400">Country:</span>
                          <span className="text-white ml-2">{vendor.country_code}</span>
                        </div>
                      )}
                      {vendor.contact_email && (
                        <div>
                          <span className="text-slate-400">Contact:</span>
                          <span className="text-white ml-2">{vendor.contact_email}</span>
                        </div>
                      )}
                      {vendor.sla_uptime_percentage && (
                        <div>
                          <span className="text-slate-400">SLA Uptime:</span>
                          <span className="text-white ml-2">{vendor.sla_uptime_percentage}%</span>
                        </div>
                      )}
                      {vendor.last_reviewed_at && (
                        <div>
                          <span className="text-slate-400">Last Reviewed:</span>
                          <span className="text-white ml-2">{format(new Date(vendor.last_reviewed_at), "PP")}</span>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="bg-slate-800/50 rounded-lg p-8 border border-slate-700 text-center">
                <Building2 className="mx-auto text-slate-500 mb-4" size={48} />
                <p className="text-slate-400">No vendors registered</p>
              </div>
            )}
          </div>
        )}

        {activeTab === "sla" && (
          <div className="space-y-4">
            {slasLoading ? (
              <div className="text-slate-400">Loading SLA monitoring...</div>
            ) : slas && slas.length > 0 ? (
              <div className="space-y-4">
                {slas.map((sla) => (
                  <div key={sla.id} className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <h3 className="text-white font-semibold mb-1">{sla.service_name}</h3>
                        <span className="text-slate-400 text-sm">{sla.service_type}</span>
                      </div>
                      {sla.sla_met ? (
                        <CheckCircle className="text-emerald-400" size={24} />
                      ) : (
                        <XCircle className="text-red-400" size={24} />
                      )}
                    </div>
                    <div className="grid grid-cols-2 gap-4 mb-4">
                      <div>
                        <span className="text-slate-400 text-sm">Target Uptime:</span>
                        <div className="text-white font-semibold">{sla.sla_target_uptime}%</div>
                      </div>
                      {sla.actual_uptime !== null && (
                        <div>
                          <span className="text-slate-400 text-sm">Actual Uptime:</span>
                          <div className={`font-semibold ${
                            sla.actual_uptime >= sla.sla_target_uptime ? "text-emerald-400" : "text-red-400"
                          }`}>
                            {sla.actual_uptime.toFixed(2)}%
                          </div>
                        </div>
                      )}
                      <div>
                        <span className="text-slate-400 text-sm">Downtime:</span>
                        <div className="text-white font-semibold">{sla.downtime_minutes} minutes</div>
                      </div>
                      <div>
                        <span className="text-slate-400 text-sm">Incidents:</span>
                        <div className="text-white font-semibold">{sla.incidents_count}</div>
                      </div>
                    </div>
                    <div className="text-sm text-slate-400">
                      <div>
                        Period: {format(new Date(sla.monitoring_period_start), "PP")} - {format(new Date(sla.monitoring_period_end), "PP")}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="bg-slate-800/50 rounded-lg p-8 border border-slate-700 text-center">
                <Activity className="mx-auto text-slate-500 mb-4" size={48} />
                <p className="text-slate-400">No SLA monitoring configured</p>
              </div>
            )}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}

