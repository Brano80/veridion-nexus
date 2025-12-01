"use client";

import { useState, useEffect } from 'react';

interface LogEntry {
  timestamp: string;
  action_summary: string;
  seal_id: string;
  status: string;
}

export default function Home() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  // NOVÝ STATE PRE KILL SWITCH
  const [isRevoked, setIsRevoked] = useState(false);

  // Fetch logs from Rust API on load
  useEffect(() => {
    const fetchLogs = async () => {
      try {
        const res = await fetch('http://127.0.0.1:8080/logs');
        if (res.ok) {
          const data = await res.json();
          // Reverse to show most recent logs at the top
          setLogs([...data].reverse());
        }
      } catch (error) {
        console.error("Failed to fetch logs:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchLogs();
    const interval = setInterval(fetchLogs, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleDownload = () => {
    window.open('http://127.0.0.1:8080/download_report', '_blank');
  };

  // NOVÁ FUNKCIA PRE VIDEO EFEKT - VOLÁ BACKEND API
  const handleRevoke = async () => {
    if (confirm("⚠️ CRITICAL WARNING: Are you sure you want to revoke all Identity Keys for this Agent? This will stop all operations immediately.")) {
        try {
            const response = await fetch('http://127.0.0.1:8080/revoke_keys', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            });
            
            if (response.ok) {
                setIsRevoked(true);
                alert("✅ SUCCESS: Identity Certificates Revoked via Signicat API. Agent is now isolated.");
            } else {
                alert("❌ ERROR: Failed to revoke keys. Please try again.");
            }
        } catch (error) {
            console.error("Failed to revoke keys:", error);
            alert("❌ ERROR: Failed to connect to API. Please check if the server is running.");
        }
    }
  };

  return (
    <main className="min-h-screen bg-slate-950 text-slate-200 p-10 font-mono">
      {/* Header */}
      <div className="flex justify-between items-center mb-10 border-b border-slate-800 pb-4">
        <div>
          <h1 className="text-3xl font-bold text-emerald-400 tracking-wider">VERIDION NEXUS</h1>
          <p className="text-slate-500 text-sm mt-1">Sovereign Governance Console | v1.0.0</p>
        </div>
        <div className="flex gap-4 items-center">
          
          {/* STATUS BAR - MENÍ FARBU KEĎ JE REVOKED */}
          <div className={`flex items-center gap-2 px-3 py-1 border rounded text-xs transition-colors ${
              isRevoked 
              ? "bg-red-900/30 border-red-800 text-red-500" 
              : "bg-emerald-900/30 border-emerald-800 text-emerald-400"
          }`}>
            <span className={`w-2 h-2 rounded-full ${isRevoked ? "bg-red-500" : "bg-emerald-500 animate-pulse"}`}></span>
            {isRevoked ? "SYSTEM LOCKDOWN" : "SYSTEM OPERATIONAL"}
          </div>
          
          <button 
            onClick={handleDownload}
            className="bg-blue-900/50 hover:bg-blue-800/80 text-blue-400 border border-blue-800 px-4 py-2 rounded text-sm font-bold transition-all flex items-center gap-2"
          >
            📄 DOWNLOAD ANNEX IV
          </button>

          {/* KILL SWITCH TLAČIDLO */}
          <button 
            onClick={handleRevoke}
            disabled={isRevoked}
            className={`px-6 py-2 rounded text-sm font-bold transition-all border ${
                isRevoked
                ? "bg-slate-800 border-slate-700 text-slate-500 cursor-not-allowed"
                : "bg-red-900/50 hover:bg-red-800/80 text-red-400 border border-red-800"
            }`}
          >
            {isRevoked ? "⛔ KEYS REVOKED" : "REVOKE AGENT KEYS"}
          </button>
        </div>
      </div>

      {/* Data Table */}
      <div className={`bg-slate-900 border rounded-lg overflow-hidden shadow-2xl transition-all ${isRevoked ? "border-red-900 opacity-50 grayscale" : "border-slate-800"}`}>
        <table className="w-full text-left text-sm">
          <thead className="bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800">
            <tr>
              <th className="p-4">Timestamp</th>
              <th className="p-4">Agent Action</th>
              <th className="p-4">Qualified Seal ID (eIDAS)</th>
              <th className="p-4">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800">
            {logs.length === 0 ? (
              <tr>
                <td colSpan={4} className="p-8 text-center text-slate-600">
                  {loading ? "Connecting to Neural Nexus..." : "No compliance records found."}
                </td>
              </tr>
            ) : (
              logs.map((log, i) => (
                <tr key={i} className="hover:bg-slate-800/50 transition-colors">
                  <td className="p-4 text-slate-400 font-mono">{log.timestamp}</td>
                  <td className="p-4 font-medium text-white">{log.action_summary}</td>
                  <td className="p-4 font-mono text-xs text-blue-400">{log.seal_id}</td>
                  <td className="p-4">
                    <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-emerald-950 text-emerald-400 text-xs font-bold border border-emerald-900">
                      ✓ {log.status}
                    </span>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </main>
  );
}
