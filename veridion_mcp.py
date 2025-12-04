from fastmcp import FastMCP
import httpx
import os

mcp = FastMCP(name="Veridion Nexus Integration")

VERIDION_API_URL = "http://localhost:8080"

@mcp.tool()
async def secure_compliance_seal(agent_id: str, action_type: str, sensitive_data: str) -> str:
    # Bezpečný print bez emoji pre Windows
    print(f"[MCP] Request received from: {agent_id}")

    async with httpx.AsyncClient() as client:
        try:
            response = await client.post(
                f"{VERIDION_API_URL}/log_action",
                json={
                    "agent_id": agent_id,
                    "action": action_type,
                    "payload": sensitive_data,
                    "target_region": "EU"
                },
                timeout=5.0
            )
            
            if response.status_code == 200:
                data = response.json()
                # Odstránené emoji "✅" pre istotu, nahradené textom [OK]
                return (f"[OK] COMPLIANCE SUCCESS. Action Sealed.\n"
                        f"Seal ID: {data.get('seal_id')}\n"
                        f"Tx ID: {data.get('tx_id')}")
            
            elif response.status_code == 403:
                return "[BLOCKED] CRITICAL FAILURE: Sovereign Lock Violation."
            
            else:
                return f"System Error: {response.text}"

        except Exception as e:
            return f"Connection Error: {str(e)}"

if __name__ == "__main__":
    mcp.run()
