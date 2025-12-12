import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  // Allow API calls to localhost backend
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: "http://127.0.0.1:8080/api/:path*",
      },
    ];
  },
  // Suppress React DevTools warnings about params Promise serialization
  // This is a known issue in Next.js 16 where DevTools tries to serialize props
  reactStrictMode: true,
  experimental: {
    // This helps with Next.js 16 params Promise handling in DevTools
    serverActions: {
      bodySizeLimit: '2mb',
    },
  },
};

export default nextConfig;
