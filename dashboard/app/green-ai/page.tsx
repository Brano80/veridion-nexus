"use client";

import DashboardLayout from "../components/DashboardLayout";
import { useQuery } from "@tanstack/react-query";
import { Leaf, Zap, TrendingDown } from "lucide-react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  BarChart,
  Bar,
} from "recharts";

const API_BASE = "http://127.0.0.1:8080/api/v1";

// Placeholder - would need new endpoint for energy telemetry
async function fetchEnergyData() {
  return {
    totalEnergy: 0,
    totalCarbon: 0,
    data: [],
  };
}

export default function GreenAIPage() {
  const { data: energyData, isLoading } = useQuery({
    queryKey: ["energy-telemetry"],
    queryFn: fetchEnergyData,
  });

  if (isLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-slate-400">Loading...</div>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">
            Green AI Telemetry
          </h1>
          <p className="text-slate-400">
            EU AI Act Article 40 - Energy efficiency and carbon footprint tracking
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="p-6 bg-emerald-900/20 border border-emerald-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Zap className="text-emerald-400" size={24} />
              <div className="text-3xl font-bold text-emerald-400">
                {energyData?.totalEnergy?.toFixed(4) || "0.0000"} kWh
              </div>
            </div>
            <div className="text-sm text-slate-400">Total Energy</div>
          </div>
          <div className="p-6 bg-blue-900/20 border border-blue-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <Leaf className="text-blue-400" size={24} />
              <div className="text-3xl font-bold text-blue-400">
                {energyData?.totalCarbon?.toFixed(2) || "0.00"} g CO₂
              </div>
            </div>
            <div className="text-sm text-slate-400">Carbon Footprint</div>
          </div>
          <div className="p-6 bg-orange-900/20 border border-orange-800 rounded-lg">
            <div className="flex items-center gap-3 mb-2">
              <TrendingDown className="text-orange-400" size={24} />
              <div className="text-3xl font-bold text-orange-400">
                {energyData?.data?.length || 0}
              </div>
            </div>
            <div className="text-sm text-slate-400">Measurements</div>
          </div>
        </div>

        {/* Chart Placeholder */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <h2 className="text-xl font-bold text-white mb-4">
            Energy Consumption Over Time
          </h2>
          <div className="h-64 flex items-center justify-center text-slate-500">
            <p>Energy telemetry data visualization</p>
          </div>
        </div>

        {/* Info */}
        <div className="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <div className="text-center py-12 text-slate-500">
            <Leaf size={48} className="mx-auto mb-4 opacity-50" />
            <p>Green AI Telemetry Dashboard</p>
            <p className="text-sm mt-2">
              Track energy consumption and carbon footprint of AI operations
            </p>
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}

