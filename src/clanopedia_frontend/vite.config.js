import { fileURLToPath, URL } from "url";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";
import environment from "vite-plugin-environment";
import fs from "fs";
import path from "path";

// Function to copy .env from root if it doesn't exist in frontend
const ensureFrontendEnv = () => {
  const rootEnvPath = path.resolve(__dirname, "../../.env");
  const frontendEnvPath = path.resolve(__dirname, ".env");

  if (!fs.existsSync(frontendEnvPath) && fs.existsSync(rootEnvPath)) {
    const envContent = fs.readFileSync(rootEnvPath, "utf-8");
    // Transform the content to add VITE_ prefix where needed
    const transformedContent = envContent
      .split("\n")
      .map((line) => {
        if (line.startsWith("CANISTER_ID_") || line.startsWith("DFX_")) {
          return `VITE_${line}`;
        }
        return line;
      })
      .join("\n");
    fs.writeFileSync(frontendEnvPath, transformedContent);
  }
};

// Ensure .env exists before loading
ensureFrontendEnv();

export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current directory.
  const env = loadEnv(mode, process.cwd(), "");

  return {
    build: {
      emptyOutDir: true,
    },
    define: {
      'process.env': {}
    },
    optimizeDeps: {
      esbuildOptions: {
        define: {
          global: "globalThis",
        },
      },
    },
    server: {
      proxy: {
        "/api": {
          target: "http://127.0.0.1:4943",
          changeOrigin: true,
        },
      },
    },
    plugins: [
      react(),
      environment("all", { prefix: "CANISTER_" }),
      environment("all", { prefix: "DFX_" }),
    ],
    resolve: {
      alias: [
        {
          find: "declarations",
          replacement: fileURLToPath(
            new URL("../declarations", import.meta.url)
          ),
        },
      ],
      dedupe: ["@dfinity/agent"],
    },
  };
});
