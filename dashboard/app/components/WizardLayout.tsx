"use client";

import Link from "next/link";
import { Shield } from "lucide-react";

interface WizardLayoutProps {
  children: React.ReactNode;
}

export default function WizardLayout({ children }: WizardLayoutProps) {
  return (
    <div className="min-h-screen bg-slate-900 flex flex-col">
      {/* Simple Header - No Sidebar */}
      <header className="border-b border-slate-800 bg-slate-900">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <Link href="/" className="flex items-center gap-3 hover:opacity-80 transition-opacity">
              <Shield className="text-emerald-400" size={32} />
              <span className="text-2xl font-bold text-white">Veridion Nexus</span>
            </Link>
            <div className="text-sm text-slate-400">
              Setup Wizard
            </div>
          </div>
        </div>
      </header>

      {/* Main Content - Full Width, No Sidebar */}
      <main className="flex-1 overflow-auto">
        {children}
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800 py-4 bg-slate-900">
        <div className="max-w-7xl mx-auto px-6 text-center text-xs text-slate-500">
          <p>
            Veridion Nexus provides technical governance tools to assist with regulatory compliance. It does not constitute legal advice. You remain solely responsible for your compliance with GDPR, DORA, and the EU AI Act.
          </p>
        </div>
      </footer>
    </div>
  );
}

