"use client";

import { useQuery } from "@tanstack/react-query";

const API_BASE = "http://127.0.0.1:8080/api/v1";

export interface Module {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  category: "core" | "operational" | "integration";
  enabled: boolean;
  requires_license: boolean;
}

export function useModules() {
  return useQuery<Module[]>({
    queryKey: ["modules"],
    queryFn: async () => {
      const token = localStorage.getItem("token");
      // If no token, return empty array (wizard doesn't need modules list)
      if (!token) {
        return [];
      }
      const response = await fetch(`${API_BASE}/modules`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      if (!response.ok) {
        // Silently fail if unauthorized (wizard doesn't need auth)
        if (response.status === 401) {
          return [];
        }
        throw new Error("Failed to fetch modules");
      }
      const data = await response.json();
      return data.modules || [];
    },
    refetchInterval: 30000, // Refetch every 30 seconds
    retry: false, // Don't retry on auth errors
  });
}

export function useModuleEnabled(moduleName: string) {
  return useQuery<boolean>({
    queryKey: ["module-status", moduleName],
    queryFn: async () => {
      const response = await fetch(`${API_BASE}/modules/${moduleName}/status`);
      if (!response.ok) {
        return false;
      }
      const data = await response.json();
      return data.enabled || false;
    },
    refetchInterval: 30000,
  });
}

