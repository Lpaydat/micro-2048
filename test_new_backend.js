#!/usr/bin/env node

// Test script for new backend integration
const CHAIN_ID = "19b926701cf2480afed7b51fd210cce5c53a6cbd508eee8a70bea6253327345a";
const APPLICATION_ID = "362f22cf775609d9bfad7c8fa4969691d23fba415efb3430e3bebfec420ceeed";
const PORT = 8088;
const WEBSITE = "localhost";

console.log("🧪 Testing New Backend Integration");
console.log("==================================");
console.log(`Chain ID: ${CHAIN_ID}`);
console.log(`Application ID: ${APPLICATION_ID}`);
console.log(`GraphQL Endpoint: http://${WEBSITE}:${PORT}/graphql`);

// Test GraphQL endpoint accessibility
async function testGraphQLEndpoint() {
    try {
        const response = await fetch(`http://${WEBSITE}:${PORT}/graphql`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: `
                    query {
                        leaderboards {
                            leaderboardId
                            tournamentName
                            active
                        }
                    }
                `
            })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        console.log("\n✅ GraphQL Endpoint Response:");
        console.log(JSON.stringify(data, null, 2));
        
        return data;
    } catch (error) {
        console.error("\n❌ GraphQL Endpoint Error:");
        console.error(error.message);
        return null;
    }
}

// Test tournament creation
async function testTournamentCreation() {
    try {
        const futureTime = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now
        
        const response = await fetch(`http://${WEBSITE}:${PORT}/graphql`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: `
                    mutation CreateTournament($tournamentName: String!, $startTime: String!, $endTime: String!, $shardNumber: Int!, $baseTriggererCount: Int!) {
                        leaderboardAction(
                            tournamentName: $tournamentName,
                            startTime: $startTime,
                            endTime: $endTime,
                            shardNumber: $shardNumber,
                            baseTriggererCount: $baseTriggererCount
                        )
                    }
                `,
                variables: {
                    tournamentName: "Backend Test Tournament",
                    startTime: futureTime.toString(),
                    endTime: (futureTime + 86400).toString(), // 24 hours later
                    shardNumber: 0,
                    baseTriggererCount: 2
                }
            })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        console.log("\n✅ Tournament Creation Response:");
        console.log(JSON.stringify(data, null, 2));
        
        return data;
    } catch (error) {
        console.error("\n❌ Tournament Creation Error:");
        console.error(error.message);
        return null;
    }
}

// Run tests
async function runTests() {
    console.log("\n🔍 Testing GraphQL endpoint...");
    const endpointResult = await testGraphQLEndpoint();
    
    if (endpointResult) {
        console.log("\n🔍 Testing tournament creation...");
        await testTournamentCreation();
    }
    
    console.log("\n🏁 Test completed!");
}

// Run tests
runTests().catch(console.error);