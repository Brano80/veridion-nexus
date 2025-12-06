"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Activity, AlertCircle, CheckCircle, Clock } from "lucide-react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";

const API_BASE = "http://127.0.0.1:8080/api/v1";

async function fetchMonitoringEvents() {
  const res = await fetch(`${API_BASE}/monitoring/events`);
  return res.json();
}

export default function MonitoringPage() {
  const { data: data, isLoading } = useQuery({
    queryKey: ["monitoring-events"],
    queryFn: fetchMonitoringEvents,
  });

  const events = data?.events || [];

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading...</div>
        </div>
      </DashboardLayout>
    );
  }

  const openEvents = events.filter((e: any) => e.resolution_status === "OPEN");
  const resolvedEvents = events.filter(
    (e: any) => e.resolution_status === "RESOLVED"
  );
  const criticalEvents = events.filter((e: any) => e.severity === "CRITICAL");

  // Prepare chart data
  const chartData = events
    .slice(0, 10)
    .map((event: any) => ({
      date: event.detected_at?.split(" ")[0] || "N/A",
      events: 1,
      severity: event.severity === "CRITICAL" ? 1 : 0,
    }));

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">
            Post-Market Monitoring
          </h1>
          <p className="text-slate-400">
            EU AI Act Article 72 - Continuous monitoring and incident tracking
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="p-6 bg-orange-900/20 border border-orange-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Clock className="text-orange-400" size={24} />
              <div className="text-3xl font-bold text-orange-400">
                {openEvents.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Open Events</div>
          </div>
          <div className="p-6 bg-emerald-900/20 border border-emerald-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle className="text-emerald-400" size={24} />
              <div className="text-3xl font-bold text-emerald-400">
                {resolvedEvents.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Resolved</div>
          </div>
          <div className="p-6 bg-red-900/20 border border-red-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <AlertCircle className="text-red-400" size={24} />
              <div className="text-3xl font-bold text-red-400">
                {criticalEvents.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Critical</div>
          </div>
          <div className="p-6 bg-blue-900/20 border border-blue-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Activity className="text-blue-400" size={24} />
              <div className="text-3xl font-bold text-blue-400">
                {events.length}
              </div>
            </div>
            <div className="text-sm text-slate-400">Total Events</div>
          </div>
        </div>

        {/* Chart */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Event Timeline</h2>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="date" stroke="#9ca3af" />
              <YAxis stroke="#9ca3af" />
              <Tooltip
                contentStyle={{
                  backgroundColor: "#1e293b",
                  border: "1px solid #334155",
                  color: "#e2e8f0",
                }}
              />
              <Legend />
              <Line
                type="monotone"
                dataKey="events"
                stroke="#10b981"
                strokeWidth={2}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* Events List */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">Monitoring Events</h2>
          <div className="space-y-3">
            {events.map((event: any, i: number) => (
              <div
                key={i}
                className="p-4 bg-slate-800/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          event.severity === "CRITICAL"
                            ? "bg-red-900/30 text-red-400 border border-red-800"
                            : event.severity === "HIGH"
                            ? "bg-orange-900/30 text-orange-400 border border-orange-800"
                            : "bg-blue-900/30 text-blue-400 border border-blue-800"
                        }`}
                      >
                        {event.severity}
                      </span>
                      <span className="text-sm font-medium text-white">
                        {event.event_type}
                      </span>
                      <span
                        className={`px-2 py-1 rounded text-xs ${
                          event.resolution_status === "OPEN"
                            ? "bg-orange-900/30 text-orange-400"
                            : "bg-emerald-900/30 text-emerald-400"
                        }`}
                      >
                        {event.resolution_status}
                      </span>
                    </div>
                    <div className="text-sm text-slate-300 mb-1">
                      System: {event.system_id}
                    </div>
                    <div className="text-xs text-slate-500">
                      Detected: {event.detected_at}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {events.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              No monitoring events found
            </div>
          )}
        </div>
      </div>
    </DashboardLayout>
  );
}

