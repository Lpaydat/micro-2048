#!/usr/bin/env node

// Test script for new backend integration v2
const CHAIN_ID = "19b926701cf2480afed7b51fd210cce5c53a6cbd508eee8a70bea6253327345a";
const APPLICATION_ID = "362f22cf775609d9bfad7c8fa4969691d23fba415efb3430e3bebfec420ceeed";
const PORT = 8088;

console.log("🧪 Testing New Backend Integration v2");
console.log("=====================================");
console.log(`Chain ID: ${CHAIN_ID}`);
console.log(`Application ID: ${APPLICATION_ID}`);

// Test the chain query
async function testChainQuery() {
    try {
        const response = await fetch(`http://localhost:${PORT}/`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: `
                    query GetChain($chainId: String!) {
                        chain(chainId: $chainId) {
                            chainId
                            executionState {
                                system {
                                    description
                                }
                            }
                        }
                    }
                `,
                variables: {
                    chainId: CHAIN_ID
                }
            })
        });

        const data = await response.json();
        console.log("\n✅ Chain Query Response:");
        console.log(JSON.stringify(data, null, 2));
        return data;
    } catch (error) {
        console.error("\n❌ Chain Query Error:");
        console.error(error.message);
        return null;
    }
}

// Test application query  
async function testApplicationQuery() {
    try {
        const response = await fetch(`http://localhost:${PORT}/`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: `
                    query GetApplications($chainId: String!) {
                        applications(chainId: $chainId) {
                            id
                            description
                        }
                    }
                `,
                variables: {
                    chainId: CHAIN_ID
                }
            })
        });

        const data = await response.json();
        console.log("\n✅ Applications Query Response:");
        console.log(JSON.stringify(data, null, 2));
        return data;
    } catch (error) {
        console.error("\n❌ Applications Query Error:");
        console.error(error.message);
        return null;
    }
}

// Test if there's an application-specific endpoint
async function testApplicationEndpoint() {
    const appUrl = `http://localhost:${PORT}/chains/${CHAIN_ID}/applications/${APPLICATION_ID}`;
    console.log(`\n🔍 Testing application-specific endpoint: ${appUrl}`);
    
    try {
        const response = await fetch(appUrl);
        console.log(`Status: ${response.status} ${response.statusText}`);
        
        if (response.ok) {
            const text = await response.text();
            console.log("Response preview:", text.substring(0, 200) + "...");
        }
    } catch (error) {
        console.error("Error:", error.message);
    }
}

// Check for GraphQL at application level
async function testApplicationGraphQL() {
    const appGraphQLUrl = `http://localhost:${PORT}/chains/${CHAIN_ID}/applications/${APPLICATION_ID}/graphql`;
    console.log(`\n🔍 Testing application GraphQL: ${appGraphQLUrl}`);
    
    try {
        const response = await fetch(appGraphQLUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: "query { __schema { queryType { name } } }"
            })
        });
        
        console.log(`Status: ${response.status} ${response.statusText}`);
        
        if (response.ok) {
            const data = await response.json();
            console.log("GraphQL Schema available!");
            console.log(JSON.stringify(data, null, 2));
        }
    } catch (error) {
        console.error("Error:", error.message);
    }
}

async function runTests() {
    await testChainQuery();
    await testApplicationQuery();
    await testApplicationEndpoint();
    await testApplicationGraphQL();
    console.log("\n🏁 Tests completed!");
}

runTests().catch(console.error);