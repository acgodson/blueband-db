import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import * as dotenv from "dotenv";
import { idlFactory } from "./declarations/blueband_rust.did.js";

dotenv.config();

// Types
interface SearchResult {
  document_id: string;
  text: string;
  chunk_id: string;
  score: number;
}

interface DemoResult {
  Ok: SearchResult[];
}

// Simple test data - perfect for README
const docs = [
  "Pizza is a delicious Italian food with cheese and tomatoes",
  "Soccer is the most popular sport in the world",
  "JavaScript is a programming language for web development",
  "Sushi is a traditional Japanese dish with rice and fish",
  "Basketball was invented in America and is played worldwide",
];

class SimpleTest {
  private agent: HttpAgent;
  private actor: any;
  private proxyUrl: string;

  constructor() {
    this.proxyUrl = process.env.PROXY_URL || "";

    // Create agent with identity
    const identity = Ed25519KeyIdentity.generate();

    this.agent = new HttpAgent({
      host: process.env.HOST || "http://127.0.0.1:4943",
      identity,
    });

    // Fetch root key for local replica
    this.agent.fetchRootKey().catch(console.error);

    this.actor = Actor.createActor(idlFactory, {
      agent: this.agent,
      canisterId: Principal.fromText(process.env.CANISTER_ID),
    });
  }

  async testQuery(query: string, expectedType: string) {
    console.log(`🔍 Query: "${query}"`);
    console.log(`Expected: ${expectedType}`);
    console.log("─".repeat(50));

    try {
      const result = (await this.actor.demo_vector_similarity(
        docs,
        query,
        this.proxyUrl,
        [1], // Only return top result
        []
      )) as DemoResult;

      if ("Ok" in result && result.Ok.length > 0) {
        const topResult = result.Ok[0];
        const score = (topResult.score * 100).toFixed(1);
        console.log(`✅ [${score}%] ${topResult.text}`);
      } else {
        console.log("❌ No results found");
      }
    } catch (error) {
      console.error("❌ Error:", error);
    }
    console.log();
  }

  async runSimpleTest() {
    console.log("🚀 Blueband Vector Search - Simple Test\n");

    await this.testQuery("What food do Italians eat?", "Italian food");
    await this.testQuery("Which sport is most popular?", "Sports");
    await this.testQuery("What language builds websites?", "Programming");
    await this.testQuery("Traditional Japanese food?", "Japanese cuisine");

    console.log("✨ Test complete! Your vector search is working.");
  }
}

// Run the simple test
const test = new SimpleTest();
test.runSimpleTest().catch(console.error);
