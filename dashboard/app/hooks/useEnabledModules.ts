"use client";

import { useQuery } from "@tanstack/react-query";

const API_BASE = "http://127.0.0.1:8080/api/v1";

export interface EnabledModule {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  category: "core" | "operational" | "integration";
  enabled: boolean;
  requires_license: boolean;
}

export function useEnabledModules() {
  return useQuery<EnabledModule[]>({
    queryKey: ["enabled-modules"],
    queryFn: async () => {
      const token = localStorage.getItem("token");
      if (!token) {
        return [];
      }
      const response = await fetch(`${API_BASE}/my/enabled-modules`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      if (!response.ok) {
        if (response.status === 401) {
          return [];
        }
        throw new Error("Failed to fetch enabled modules");
      }
      const data = await response.json();
      return data.modules || [];
    },
    refetchInterval: 30000, // Refetch every 30 seconds
    retry: false,
  });
}

