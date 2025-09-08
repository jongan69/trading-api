#!/usr/bin/env python3
"""
Test script for Trading API MCP Server

This script tests the MCP server functionality by simulating tool calls
and verifying responses. Run this after starting both the Rust API and MCP server.
"""

import asyncio
import json
import sys
from typing import Any, Dict

import httpx


class MCPTester:
    """Simple tester for MCP server functionality"""
    
    def __init__(self, api_base_url: str = "http://localhost:3000"):
        self.api_base_url = api_base_url
        self.client = httpx.AsyncClient(timeout=30.0)
    
    async def test_api_connection(self) -> bool:
        """Test if the Rust API is running and accessible"""
        try:
            response = await self.client.get(f"{self.api_base_url}/health")
            if response.status_code == 200:
                print("✅ Rust API is running and accessible")
                return True
            else:
                print(f"❌ Rust API returned status {response.status_code}")
                return False
        except Exception as e:
            print(f"❌ Failed to connect to Rust API: {e}")
            return False
    
    async def test_endpoints(self) -> Dict[str, bool]:
        """Test key API endpoints that the MCP server uses"""
        endpoints_to_test = [
            "/trending/stocks",
            "/trending/crypto", 
            "/news",
            "/reddit/stocks",
            "/coingecko/top",
            "/kraken/ticker"
        ]
        
        results = {}
        
        for endpoint in endpoints_to_test:
            try:
                response = await self.client.get(f"{self.api_base_url}{endpoint}")
                if response.status_code == 200:
                    print(f"✅ {endpoint} - OK")
                    results[endpoint] = True
                else:
                    print(f"⚠️  {endpoint} - Status {response.status_code}")
                    results[endpoint] = False
            except Exception as e:
                print(f"❌ {endpoint} - Error: {e}")
                results[endpoint] = False
        
        return results
    
    async def simulate_mcp_tools(self) -> Dict[str, Any]:
        """Simulate MCP tool calls to test functionality"""
        print("\n📋 Simulating MCP tool calls...")
        
        tool_tests = {
            "get_trending_stocks": {
                "endpoint": "/trending/stocks",
                "params": {"limit": 5}
            },
            "get_trending_crypto": {
                "endpoint": "/trending/crypto", 
                "params": {"limit": 5}
            },
            "get_market_news": {
                "endpoint": "/news",
                "params": {"limit": 10}
            },
            "get_reddit_trending_stocks": {
                "endpoint": "/reddit/stocks",
                "params": {"limit": 5}
            }
        }
        
        results = {}
        
        for tool_name, test_config in tool_tests.items():
            try:
                response = await self.client.get(
                    f"{self.api_base_url}{test_config['endpoint']}",
                    params=test_config['params']
                )
                
                if response.status_code == 200:
                    data = response.json()
                    print(f"✅ {tool_name} - Retrieved {len(str(data))} chars of data")
                    results[tool_name] = {
                        "success": True,
                        "data_size": len(str(data)),
                        "sample": str(data)[:200] + "..." if len(str(data)) > 200 else str(data)
                    }
                else:
                    print(f"❌ {tool_name} - Status {response.status_code}")
                    results[tool_name] = {"success": False, "error": f"HTTP {response.status_code}"}
                    
            except Exception as e:
                print(f"❌ {tool_name} - Error: {e}")
                results[tool_name] = {"success": False, "error": str(e)}
        
        return results
    
    async def run_tests(self) -> Dict[str, Any]:
        """Run all tests and return results"""
        print("🧪 Testing Trading API MCP Server Setup\n")
        
        # Test API connection
        api_connected = await self.test_api_connection()
        if not api_connected:
            return {
                "api_connection": False,
                "message": "Cannot proceed - Rust API is not accessible"
            }
        
        print("\n🔍 Testing API endpoints...")
        endpoint_results = await self.test_endpoints()
        
        # Test MCP tool simulation
        tool_results = await self.simulate_mcp_tools()
        
        # Summary
        successful_endpoints = sum(1 for success in endpoint_results.values() if success)
        total_endpoints = len(endpoint_results)
        
        successful_tools = sum(1 for result in tool_results.values() if result.get("success", False))
        total_tools = len(tool_results)
        
        print(f"\n📊 Test Summary:")
        print(f"   API Endpoints: {successful_endpoints}/{total_endpoints} working")
        print(f"   MCP Tools: {successful_tools}/{total_tools} working")
        
        if successful_endpoints == total_endpoints and successful_tools == total_tools:
            print("🎉 All tests passed! MCP server should work correctly.")
        else:
            print("⚠️  Some tests failed. Check the output above for details.")
        
        return {
            "api_connection": api_connected,
            "endpoint_results": endpoint_results,
            "tool_results": tool_results,
            "summary": {
                "endpoints_working": f"{successful_endpoints}/{total_endpoints}",
                "tools_working": f"{successful_tools}/{total_tools}",
                "all_tests_passed": successful_endpoints == total_endpoints and successful_tools == total_tools
            }
        }
    
    async def close(self):
        """Close the HTTP client"""
        await self.client.aclose()


async def main():
    """Main test function"""
    if len(sys.argv) > 1:
        api_url = sys.argv[1]
    else:
        api_url = "http://localhost:3000"
    
    print(f"Testing API at: {api_url}")
    
    tester = MCPTester(api_url)
    
    try:
        results = await tester.run_tests()
        
        # Optionally save results to file
        with open("mcp_test_results.json", "w") as f:
            json.dump(results, f, indent=2)
        print(f"\n💾 Test results saved to mcp_test_results.json")
        
    except KeyboardInterrupt:
        print("\n⏹️  Tests interrupted by user")
    except Exception as e:
        print(f"\n💥 Test failed with error: {e}")
    finally:
        await tester.close()


if __name__ == "__main__":
    asyncio.run(main())
