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
  const [downloading, setDownloading] = useState(false);

  // Fetch logs from Rust API on load
  useEffect(() => {
    const fetchLogs = async () => {
      try {
        const res = await fetch('http://127.0.0.1:8080/logs');
        if (res.ok) {
          const data = await res.json();
          setLogs(data);
        }
      } catch (error) {
        console.error("Failed to fetch logs:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchLogs();

    // Set up a live poll every 2 seconds to see updates instantly
    const interval = setInterval(fetchLogs, 2000);

    return () => clearInterval(interval);
  }, []);

  // Handle PDF report download
  const handleDownloadReport = async () => {
    try {
      setDownloading(true);
      const response = await fetch('http://127.0.0.1:8080/download_report');
      
      if (!response.ok) {
        throw new Error('Failed to download report');
      }
      
      // Get the blob from response
      const blob = await response.blob();
      
      // Create a download link
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'Veridion_Annex_IV.pdf';
      document.body.appendChild(a);
      a.click();
      
      // Cleanup
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (error) {
      console.error('Failed to download report:', error);
      alert('Failed to download report. Please try again.');
    } finally {
      setDownloading(false);
    }
  };

  return (
    <main className="min-h-screen bg-slate-950 text-slate-200 p-10 font-mono">
      {/* --- RESTORED HEADER START --- */}
      <div className="flex justify-between items-center mb-10 border-b border-slate-800 pb-4">
        <div>
          <h1 className="text-3xl font-bold text-emerald-400 tracking-wider">VERIDION NEXUS</h1>
          <p className="text-slate-500 text-sm mt-1">Sovereign Governance Console | v1.0.0</p>
        </div>
        <div className="flex gap-4 items-center">
          <div className="flex items-center gap-2 px-3 py-1 bg-emerald-900/30 border border-emerald-800 rounded text-xs text-emerald-400">
            <span className="w-2 h-2 bg-emerald-500 rounded-full animate-pulse"></span>
            SYSTEM OPERATIONAL
          </div>
          <button 
            onClick={handleDownloadReport}
            disabled={downloading}
            className="bg-blue-900/50 hover:bg-blue-800/80 disabled:opacity-50 disabled:cursor-not-allowed text-blue-400 border border-blue-800 px-6 py-2 rounded text-sm font-bold transition-all"
          >
            {downloading ? 'DOWNLOADING...' : '📄 DOWNLOAD REPORT'}
          </button>
          <button className="bg-red-900/50 hover:bg-red-800/80 text-red-400 border border-red-800 px-6 py-2 rounded text-sm font-bold transition-all">
            REVOKE AGENT KEYS
          </button>
        </div>
      </div>
      {/* --- RESTORED HEADER END --- */}

      {/* Data Table */}
      <div className="bg-slate-900 border border-slate-800 rounded-lg overflow-hidden shadow-2xl">
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
