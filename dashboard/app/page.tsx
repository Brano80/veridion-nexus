"use client";

import Link from "next/link";
import { Shield, Sparkles, CheckCircle2, ArrowRight, Code } from "lucide-react";

export default function LandingPage() {
  return (
    <div className="min-h-screen bg-slate-900 flex flex-col">
      {/* Header */}
      <header className="border-b border-slate-800">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Shield className="text-emerald-400" size={32} />
              <span className="text-2xl font-bold text-white">Veridion Nexus</span>
            </div>
            <nav className="flex items-center gap-6">
              <a
                href="/login"
                className="text-slate-400 hover:text-white transition-colors"
              >
                Login
              </a>
            </nav>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <main className="flex-1 flex items-center justify-center px-6 py-20">
        <div className="max-w-4xl mx-auto text-center space-y-8">
          {/* Main Heading */}
          <div className="space-y-4">
            <h1 className="text-5xl md:text-6xl font-bold text-white leading-tight">
              EU Compliance
              <br />
              <span className="text-emerald-400">Made Simple</span>
            </h1>
            <p className="text-xl text-slate-400 max-w-2xl mx-auto">
              Automated compliance platform for GDPR, EU AI Act, and DORA regulations.
              Configure your compliance package in minutes and start protecting your business.
            </p>
          </div>

          {/* CTA Button */}
          <div className="pt-4">
            <Link
              href="/wizard"
              className="group inline-flex items-center gap-3 px-8 py-4 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold rounded-lg transition-all duration-200 shadow-lg shadow-emerald-900/50 hover:shadow-xl hover:shadow-emerald-900/50 hover:scale-105"
            >
              <Sparkles className="group-hover:rotate-12 transition-transform" size={24} />
              <span className="text-lg">Configure your compliance packet</span>
              <ArrowRight className="group-hover:translate-x-1 transition-transform" size={20} />
            </Link>
          </div>

          {/* Features Grid */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 pt-16">
            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 text-left">
              <div className="w-12 h-12 bg-emerald-900/30 border border-emerald-800 rounded-lg flex items-center justify-center mb-4">
                <Shield className="text-emerald-400" size={24} />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">
                GDPR Compliance
              </h3>
              <p className="text-slate-400 text-sm">
                Automated data subject rights, processing restrictions, and consent management.
              </p>
            </div>

            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 text-left">
              <div className="w-12 h-12 bg-blue-900/30 border border-blue-800 rounded-lg flex items-center justify-center mb-4">
                <Sparkles className="text-blue-400" size={24} />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">
                EU AI Act Ready
              </h3>
              <p className="text-slate-400 text-sm">
                Human oversight, risk assessment, and transparency requirements for AI systems.
              </p>
            </div>

            <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 text-left">
              <div className="w-12 h-12 bg-purple-900/30 border border-purple-800 rounded-lg flex items-center justify-center mb-4">
                <CheckCircle2 className="text-purple-400" size={24} />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">
                DORA Compliance
              </h3>
              <p className="text-slate-400 text-sm">
                Operational resilience, risk management, and incident reporting for financial entities.
              </p>
            </div>
          </div>

          {/* Trust Indicators */}
          <div className="pt-12 border-t border-slate-800">
            <p className="text-sm text-slate-500 mb-4">
              Trusted by organizations across the EU
            </p>
            <div className="flex items-center justify-center gap-8 text-slate-600">
              <div className="text-sm">GDPR Compliant</div>
              <div className="text-sm">EU AI Act Ready</div>
              <div className="text-sm">DORA Certified</div>
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800 py-6">
        <div className="max-w-7xl mx-auto px-6 text-center text-xs text-slate-500 space-y-2">
          <p>
            Veridion Nexus provides technical governance tools to assist with regulatory compliance. It does not constitute legal advice. You remain solely responsible for your compliance with GDPR, DORA, and the EU AI Act.
          </p>
          <p>© 2024 Veridion Nexus. All rights reserved.</p>
        </div>
      </footer>

      {/* Developer Button - Floating */}
      <Link
        href="/dashboard"
        className="fixed bottom-6 right-6 flex items-center gap-2 px-4 py-2 bg-slate-800/80 hover:bg-slate-700/80 text-slate-300 hover:text-white border border-slate-700 rounded-lg transition-all duration-200 shadow-lg backdrop-blur-sm text-sm font-medium z-50"
        title="Developer Access"
      >
        <Code size={16} />
        <span>Developer</span>
      </Link>
    </div>
  );
}
