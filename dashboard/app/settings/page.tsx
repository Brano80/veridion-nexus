"use client";

import DashboardLayout from "../components/DashboardLayout";
import { Settings, Shield, Key, Database, Bell } from "lucide-react";

export default function SettingsPage() {
  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Settings</h1>
          <p className="text-slate-400">
            System configuration and preferences
          </p>
        </div>

        {/* Settings Sections */}
        <div className="space-y-6">
          {/* System Status */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <Shield className="text-emerald-400" size={24} />
              <h2 className="text-xl font-bold text-white">System Status</h2>
            </div>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-slate-800/50 rounded">
                <span className="text-slate-300">System Status</span>
                <span className="px-3 py-1 bg-emerald-900/30 text-emerald-400 border border-emerald-800 rounded text-sm font-medium">
                  OPERATIONAL
                </span>
              </div>
              <div className="flex items-center justify-between p-3 bg-slate-800/50 rounded">
                <span className="text-slate-300">Database Connection</span>
                <span className="px-3 py-1 bg-emerald-900/30 text-emerald-400 border border-emerald-800 rounded text-sm font-medium">
                  CONNECTED
                </span>
              </div>
            </div>
          </div>

          {/* API Configuration */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <Key className="text-blue-400" size={24} />
              <h2 className="text-xl font-bold text-white">API Configuration</h2>
            </div>
            <div className="space-y-3">
              <div className="p-3 bg-slate-800/50 rounded">
                <label className="text-sm text-slate-400 mb-1 block">
                  API Base URL
                </label>
                <input
                  type="text"
                  defaultValue="http://127.0.0.1:8080/api/v1"
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
                  readOnly
                />
              </div>
            </div>
          </div>

          {/* Database Info */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <Database className="text-purple-400" size={24} />
              <h2 className="text-xl font-bold text-white">Database</h2>
            </div>
            <div className="space-y-3">
              <div className="p-3 bg-slate-800/50 rounded">
                <span className="text-sm text-slate-400">Database Type: </span>
                <span className="text-white">PostgreSQL</span>
              </div>
              <div className="p-3 bg-slate-800/50 rounded">
                <span className="text-sm text-slate-400">Connection: </span>
                <span className="text-white">Active</span>
              </div>
            </div>
          </div>

          {/* Notifications */}
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-3 mb-4">
              <Bell className="text-orange-400" size={24} />
              <h2 className="text-xl font-bold text-white">Notifications</h2>
            </div>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-slate-800/50 rounded">
                <span className="text-slate-300">Real-time Updates</span>
                <span className="px-3 py-1 bg-emerald-900/30 text-emerald-400 border border-emerald-800 rounded text-sm font-medium">
                  ENABLED
                </span>
              </div>
              <div className="flex items-center justify-between p-3 bg-slate-800/50 rounded">
                <span className="text-slate-300">Refresh Interval</span>
                <span className="text-white">10 seconds</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

