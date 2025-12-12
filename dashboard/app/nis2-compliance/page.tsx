"use client";

import DashboardLayout from "../components/DashboardLayout";
import { Scale, AlertCircle } from "lucide-react";

export default function NIS2CompliancePage() {
  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        <div className="flex items-center gap-3">
          <Scale className="text-blue-400" size={32} />
          <h1 className="text-3xl font-bold text-slate-100">NIS2 Compliance</h1>
        </div>
        
        <div className="bg-blue-900/20 border border-blue-800 rounded-lg p-6">
          <div className="flex items-start gap-3">
            <AlertCircle className="text-blue-400 mt-0.5" size={20} />
            <div>
              <h3 className="font-semibold text-blue-400 mb-2">Enterprise Feature - Coming Soon</h3>
              <p className="text-blue-300 text-sm">
                NIS2 (Network and Information Systems Directive 2) compliance reporting will include:
              </p>
              <ul className="list-disc list-inside text-blue-300/80 text-sm mt-2 space-y-1">
                <li>Article 20: Management body accountability</li>
                <li>Article 21: Baseline cybersecurity measures (all 10 minimum measures)</li>
                <li>Article 23: Early warning incident reporting</li>
                <li>Liability reduction metrics and executive assurance reports</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

