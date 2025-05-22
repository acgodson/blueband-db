import { useState, useEffect } from "react";
import { useAuth } from "./useAuth";
import { Principal } from "@dfinity/principal";

export interface Collection {
  id: string;
  name: string;
  principal?: string;
  documentCount: number;
  createdAt: Date;
  hasToken?: boolean;
}

const STORAGE_KEY = "clanopedia_collections";
const TEST_TOKEN_PRINCIPAL = import.meta.env.VITE_CANISTER_ID_TEST_TOKEN;

export const useCollections = () => {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { bluebandActor } = useAuth();

  // Load collections from localStorage on mount
  useEffect(() => {
    const storedCollections = localStorage.getItem(STORAGE_KEY);
    if (storedCollections) {
      const parsedCollections = JSON.parse(storedCollections).map(
        (col: any) => ({
          ...col,
          createdAt: new Date(col.createdAt),
        })
      );
      setCollections(parsedCollections);
    }
  }, []);

  // Save collections to localStorage whenever they change
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(collections));
  }, [collections]);

  const createCollection = async (
    name: string,
    useToken: boolean = false
  ): Promise<boolean> => {
    if (!bluebandActor) {
      setError("Not connected to backend");
      return false;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Create collection ID from name
      const collectionId = name.toLowerCase().replace(/\s+/g, "-");

      // Prepare token canister parameter
      const tokenCanister: [] | [Principal] = useToken
        ? [Principal.fromText(TEST_TOKEN_PRINCIPAL)]
        : [];

      // Call the backend to create the collection with default settings
      const result = await bluebandActor.createCollection(
        collectionId,
        tokenCanister,
        [], // voting_threshold - will use default (51%)
        [], // quorum_threshold - will use default (10%)
        [] // expiry_time - will use default (7 days)
      );

      if (result) {
        // Add the new collection to state
        const newCollection: Collection = {
          id: collectionId,
          name,
          principal: result.toString(),
          documentCount: 0,
          createdAt: new Date(),
          hasToken: useToken,
        };

        // Update state and localStorage
        setCollections((prev) => {
          const updated = [...prev, newCollection];
          localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
          return updated;
        });

        // Wait for state to be updated
        await new Promise((resolve) => setTimeout(resolve, 100));
        return true;
      } else {
        setError("Failed to create collection");
        return false;
      }
    } catch (err) {
      console.error("Error creating collection:", err);
      setError("Failed to create collection. Please try again.");
      return false;
    } finally {
      setIsLoading(false);
    }
  };

  const removeCollection = (id: string) => {
    setCollections((prev) => prev.filter((c) => c.id !== id));
  };

  return {
    collections,
    isLoading,
    error,
    createCollection,
    removeCollection,
  };
};
