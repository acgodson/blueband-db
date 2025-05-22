import { useAuthClient } from "@dfinity/use-auth-client";
import { createActor as createBluebandActor } from "../../../declarations/blueband-db";
import { createActor as createTokenActor } from "../../../declarations/test_token";
import { useMemo, useEffect } from "react";
import { HttpAgent } from "@dfinity/agent";

export const useAuth = () => {
  const { authClient, isAuthenticated, identity, login, logout } =
    useAuthClient({
      loginOptions: {
        identityProvider:
          import.meta.env.VITE_DFX_NETWORK === "local"
            ? `http://${
                import.meta.env.VITE_CANISTER_ID_INTERNET_IDENTITY
              }.localhost:4943`
            : "https://identity.ic0.app",
        maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000),
      },
    });

  // Create authenticated actors using useMemo
  const bluebandActor = useMemo(() => {
    if (!identity) return null;

    const agent = new HttpAgent({
      identity,
      host:
        import.meta.env.VITE_DFX_NETWORK === "local"
          ? "http://localhost:4943"
          : "https://ic0.app",
    });

    // IMPORTANT: Fetch root key for local development
    if (import.meta.env.VITE_DFX_NETWORK === "local") {
      agent.fetchRootKey().catch(console.error);
    }

    return createBluebandActor(import.meta.env.VITE_CANISTER_ID_BLUEBAND_DB, {
      agent,
    });
  }, [identity, import.meta.env.VITE_DFX_NETWORK, import.meta.env.VITE_CANISTER_ID_BLUEBAND_DB]);

  const tokenActor = useMemo(() => {
    if (!identity) return null;

    const agent = new HttpAgent({
      identity,
      host:
        import.meta.env.VITE_DFX_NETWORK === "local"
          ? "http://localhost:4943"
          : "https://ic0.app",
    });

    // IMPORTANT: Fetch root key for local development
    if (import.meta.env.VITE_DFX_NETWORK === "local") {
      agent.fetchRootKey().catch(console.error);
    }

    return createTokenActor(import.meta.env.VITE_CANISTER_ID_TEST_TOKEN, {
      agent,
    });
  }, [identity, import.meta.env.VITE_DFX_NETWORK, import.meta.env.VITE_CANISTER_ID_TEST_TOKEN]);

  return {
    authClient,
    isAuthenticated,
    identity,
    principal: identity?.getPrincipal(),
    login,
    logout,
    bluebandActor,
    tokenActor,
  };
};
