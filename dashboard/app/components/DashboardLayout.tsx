"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  FileText,
  Users,
  Eye,
  AlertTriangle,
  Shield,
  ClipboardCheck,
  Calendar,
  Activity,
  Package,
  Leaf,
  Webhook,
  Settings,
  Menu,
  X,
  ScrollText,
  Trash2,
  Lock,
  Ban,
  Bot,
} from "lucide-react";
import { useState } from "react";
import { useModules } from "../hooks/useModules";

// Core Compliance Hub navigation (always visible)
const coreNavigation = [
  { name: "Compliance Overview", href: "/", icon: LayoutDashboard, module: null },
  { name: "Runtime Logs", href: "/runtime-logs", icon: ScrollText, module: null },
  { name: "Human Oversight", href: "/human-oversight", icon: Eye, module: "module_human_oversight" },
  { name: "Data Shredding", href: "/data-subjects", icon: Trash2, module: "module_data_subject_rights" },
  { name: "Processing Restrictions", href: "/processing-restrictions", icon: Lock, module: "module_data_subject_rights" },
  { name: "Processing Objections", href: "/processing-objections", icon: Ban, module: "module_data_subject_rights" },
  { name: "Automated Decisions", href: "/automated-decisions", icon: Bot, module: "module_data_subject_rights" },
  { name: "Audit & Reports", href: "/audit-reports", icon: FileText, module: null },
  { name: "Settings", href: "/settings", icon: Settings, module: null },
];

// Plugin navigation (module-dependent)
const pluginNavigation = [
  { name: "Risk Assessment", href: "/risk-assessment", icon: AlertTriangle, module: "module_risk_assessment" },
  { name: "Data Breaches", href: "/data-breaches", icon: Shield, module: "module_breach_management" },
  { name: "Consent Management", href: "/consent", icon: ClipboardCheck, module: "module_consent" },
  { name: "DPIA Tracking", href: "/dpia", icon: FileText, module: "module_dpia" },
  { name: "Retention Policies", href: "/retention", icon: Calendar, module: "module_retention" },
  { name: "Post-Market Monitoring", href: "/monitoring", icon: Activity, module: "module_monitoring" },
  { name: "AI-BOM", href: "/ai-bom", icon: Package, module: "module_ai_bom" },
  { name: "Green AI", href: "/green-ai", icon: Leaf, module: "module_green_ai" },
  { name: "Webhooks", href: "/webhooks", icon: Webhook, module: "integration_webhooks" },
];

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const pathname = usePathname();
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const { data: modules = [], isLoading: modulesLoading } = useModules();

  // DEMO MODE: Show all modules regardless of enabled status
  // For demo purposes, we want to display all available modules
  const enabledPlugins = pluginNavigation; // Show all modules in demo mode

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200">
      {/* Mobile menu button */}
      <div className="lg:hidden fixed top-4 left-4 z-50">
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          className="p-2 rounded-lg bg-slate-800 border border-slate-700"
        >
          {sidebarOpen ? <X size={20} /> : <Menu size={20} />}
        </button>
      </div>

      {/* Sidebar */}
      <aside
        className={`fixed top-0 left-0 h-full w-64 bg-slate-900 border-r border-slate-800 z-40 transition-transform duration-300 ${
          sidebarOpen ? "translate-x-0" : "-translate-x-full"
        } lg:translate-x-0`}
      >
        <div className="p-6 border-b border-slate-800">
          <h1 className="text-2xl font-bold text-emerald-400 tracking-wider">
            VERIDION NEXUS
          </h1>
          <p className="text-xs text-slate-500 mt-1">
            Compliance Dashboard v1.0.0
          </p>
        </div>

        <nav className="p-4 space-y-1 overflow-y-auto h-[calc(100vh-120px)]">
          {/* Core Compliance Hub */}
          <div className="mb-4">
            <div className="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider">
              Compliance Hub
            </div>
            {coreNavigation.map((item) => {
              const isActive = pathname === item.href;
              const Icon = item.icon;
              return (
                <Link
                  key={item.name}
                  href={item.href}
                  onClick={() => setSidebarOpen(false)}
                  className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                    isActive
                      ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                      : "text-slate-400 hover:bg-slate-800 hover:text-slate-200"
                  }`}
                >
                  <Icon size={18} />
                  <span className="text-sm font-medium">{item.name}</span>
                </Link>
              );
            })}
          </div>

          {/* Plugin Modules */}
          {enabledPlugins.length > 0 && (
            <div className="mt-6">
              <div className="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider">
                Modules
              </div>
              {enabledPlugins.map((item) => {
                const isActive = pathname === item.href;
                const Icon = item.icon;
                return (
                  <Link
                    key={item.name}
                    href={item.href}
                    onClick={() => setSidebarOpen(false)}
                    className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                      isActive
                        ? "bg-emerald-900/30 text-emerald-400 border border-emerald-800"
                        : "text-slate-400 hover:bg-slate-800 hover:text-slate-200"
                    }`}
                  >
                    <Icon size={18} />
                    <span className="text-sm font-medium">{item.name}</span>
                  </Link>
                );
              })}
            </div>
          )}
        </nav>

        <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-slate-800 bg-slate-900">
          <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-900/20 border border-emerald-800">
            <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></div>
            <span className="text-xs text-emerald-400 font-medium">
              SYSTEM OPERATIONAL
            </span>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="lg:ml-64 p-6 lg:p-10">
        {children}
      </main>

      {/* Overlay for mobile */}
      {sidebarOpen && (
        <div
          className="lg:hidden fixed inset-0 bg-black/50 z-30"
          onClick={() => setSidebarOpen(false)}
        />
      )}
    </div>
  );
}

