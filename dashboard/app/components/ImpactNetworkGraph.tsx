"use client";

import { useEffect, useRef } from "react";
import * as d3 from "d3";

interface NetworkNode {
  id: string;
  type: "agent" | "business_function" | "country" | "endpoint";
  label: string;
  value: number;
  group?: number;
}

interface NetworkLink {
  source: string;
  target: string;
  value: number;
}

interface ImpactNetworkGraphProps {
  agents: Array<{
    agent_id: string;
    business_function?: string | null;
    would_block: number;
    block_percentage: number;
    affected_endpoints: string[];
  }>;
  countries: Record<string, number>;
  endpoints?: Record<string, number>;
  width?: number;
  height?: number;
}

export default function ImpactNetworkGraph({
  agents,
  countries,
  endpoints = {},
  width = 800,
  height = 600,
}: ImpactNetworkGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || agents.length === 0) return;

    // Clear previous render
    d3.select(svgRef.current).selectAll("*").remove();

    const svg = d3.select(svgRef.current);
    const margin = { top: 20, right: 20, bottom: 20, left: 20 };
    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;

    // Create nodes and links
    const nodes: NetworkNode[] = [];
    const links: NetworkLink[] = [];

    // Add agent nodes
    agents.slice(0, 20).forEach((agent) => {
      nodes.push({
        id: `agent-${agent.agent_id}`,
        type: "agent",
        label: agent.agent_id.length > 15 ? `${agent.agent_id.substring(0, 15)}...` : agent.agent_id,
        value: agent.would_block,
        group: agent.block_percentage >= 50 ? 0 : agent.block_percentage > 0 ? 1 : 2,
      });

      // Link to business function if available
      if (agent.business_function) {
        const bfId = `bf-${agent.business_function}`;
        if (!nodes.find((n) => n.id === bfId)) {
          nodes.push({
            id: bfId,
            type: "business_function",
            label: agent.business_function,
            value: agent.would_block,
            group: 3,
          });
        }
        links.push({
          source: `agent-${agent.agent_id}`,
          target: bfId,
          value: agent.would_block,
        });
      }

      // Link to top endpoints
      agent.affected_endpoints.slice(0, 3).forEach((endpoint) => {
        const endpointId = `endpoint-${endpoint}`;
        if (!nodes.find((n) => n.id === endpointId)) {
          const endpointCount = endpoints[endpoint] || 0;
          nodes.push({
            id: endpointId,
            type: "endpoint",
            label: endpoint.length > 20 ? `${endpoint.substring(0, 20)}...` : endpoint,
            value: endpointCount,
            group: 4,
          });
        }
        links.push({
          source: `agent-${agent.agent_id}`,
          target: endpointId,
          value: agent.would_block,
        });
      });
    });

    // Add top countries
    const topCountries = Object.entries(countries)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 10);
    
    topCountries.forEach(([country, count]) => {
      const countryId = `country-${country}`;
      nodes.push({
        id: countryId,
        type: "country",
        label: country,
        value: count,
        group: country === "US" || country === "CN" || country === "RU" ? 0 : 2,
      });

      // Link countries to agents that would be blocked
      agents
        .filter((a) => a.would_block > 0)
        .slice(0, 5)
        .forEach((agent) => {
          links.push({
            source: `agent-${agent.agent_id}`,
            target: countryId,
            value: Math.min(agent.would_block, count),
          });
        });
    });

    // Create simulation
    const simulation = d3
      .forceSimulation(nodes as any)
      .force(
        "link",
        d3
          .forceLink(links)
          .id((d: any) => d.id)
          .distance((d: any) => 50 + d.value * 0.1)
      )
      .force("charge", d3.forceManyBody().strength(-300))
      .force("center", d3.forceCenter(innerWidth / 2, innerHeight / 2))
      .force("collision", d3.forceCollide().radius(30));

    // Color scale
    const colorScale = d3
      .scaleOrdinal<string>()
      .domain(["0", "1", "2", "3", "4"])
      .range(["#ef4444", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6"]);

    // Create links
    const link = svg
      .append("g")
      .attr("class", "links")
      .selectAll("line")
      .data(links)
      .enter()
      .append("line")
      .attr("stroke", "#475569")
      .attr("stroke-opacity", 0.3)
      .attr("stroke-width", (d) => Math.sqrt(d.value) * 0.5);

    // Create nodes
    const node = svg
      .append("g")
      .attr("class", "nodes")
      .selectAll("circle")
      .data(nodes)
      .enter()
      .append("circle")
      .attr("r", (d) => 5 + Math.sqrt(d.value) * 0.3)
      .attr("fill", (d) => colorScale(String(d.group || 0)))
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5)
      .call(
        d3
          .drag<SVGCircleElement, NetworkNode>()
          .on("start", dragstarted)
          .on("drag", dragged)
          .on("end", dragended) as any
      );

    // Add labels
    const labels = svg
      .append("g")
      .attr("class", "labels")
      .selectAll("text")
      .data(nodes)
      .enter()
      .append("text")
      .text((d) => d.label)
      .attr("font-size", "10px")
      .attr("fill", "#e2e8f0")
      .attr("dx", 8)
      .attr("dy", 4);

    // Tooltip
    const tooltip = d3
      .select("body")
      .append("div")
      .attr("class", "tooltip")
      .style("opacity", 0)
      .style("position", "absolute")
      .style("background", "rgba(0, 0, 0, 0.9)")
      .style("color", "#fff")
      .style("padding", "8px")
      .style("border-radius", "4px")
      .style("font-size", "12px")
      .style("pointer-events", "none")
      .style("z-index", "1000");

    node
      .on("mouseover", function (event, d) {
        tooltip.transition().duration(200).style("opacity", 0.9);
        tooltip
          .html(
            `<strong>${d.label}</strong><br/>Type: ${d.type}<br/>Value: ${d.value.toLocaleString()}`
          )
          .style("left", event.pageX + 10 + "px")
          .style("top", event.pageY - 28 + "px");
      })
      .on("mouseout", function () {
        tooltip.transition().duration(500).style("opacity", 0);
      });

    // Update positions on simulation tick
    simulation.on("tick", () => {
      link
        .attr("x1", (d: any) => d.source.x)
        .attr("y1", (d: any) => d.source.y)
        .attr("x2", (d: any) => d.target.x)
        .attr("y2", (d: any) => d.target.y);

      node.attr("cx", (d: any) => d.x).attr("cy", (d: any) => d.y);

      labels.attr("x", (d: any) => d.x).attr("y", (d: any) => d.y);
    });

    function dragstarted(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }

    function dragged(event: any, d: any) {
      d.fx = event.x;
      d.fy = event.y;
    }

    function dragended(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    }

    // Cleanup
    return () => {
      tooltip.remove();
      simulation.stop();
    };
  }, [agents, countries, endpoints, width, height]);

  return (
    <div className="w-full">
      <svg ref={svgRef} width={width} height={height} className="border border-slate-700 rounded-lg bg-slate-900" />
      <div className="mt-4 flex flex-wrap gap-4 text-xs text-slate-400">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-red-500"></div>
          <span>Critical Impact (≥50% blocked)</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
          <span>Partial Impact (1-49% blocked)</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-green-500"></div>
          <span>Safe (0% blocked)</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-blue-500"></div>
          <span>Business Function</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-purple-500"></div>
          <span>Endpoint</span>
        </div>
      </div>
    </div>
  );
}

