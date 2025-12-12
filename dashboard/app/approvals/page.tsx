"use client";

import DashboardLayout from "../components/DashboardLayout";
import { CheckCircle2, AlertCircle } from "lucide-react";

export default function ApprovalsPage() {
  return (
    <DashboardLayout>
      <div className="p-8 space-y-6">
        <div className="flex items-center gap-3">
          <CheckCircle2 className="text-blue-400" size={32} />
          <h1 className="text-3xl font-bold text-slate-100">Approval Workflow</h1>
        </div>
        
        <div className="bg-blue-900/20 border border-blue-800 rounded-lg p-6">
          <div className="flex items-start gap-3">
            <AlertCircle className="text-blue-400 mt-0.5" size={20} />
            <div>
              <h3 className="font-semibold text-blue-400 mb-2">Enterprise Feature - Coming Soon</h3>
              <p className="text-blue-300 text-sm">
                Multi-step approval workflow will include:
              </p>
              <ul className="list-disc list-inside text-blue-300/80 text-sm mt-2 space-y-1">
                <li>2-person rule for critical policies</li>
                <li>Approval queue dashboard</li>
                <li>Approval history and audit trail</li>
                <li>Role-based approval routing</li>
                <li>Approval delegation and SLA tracking</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

