import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Collection {
    id: string;
    name: string;
    bucketId: string;
    createdAt: Date;
    documentCount: number;
}

interface CollectionsState {
    collections: Collection[];
    selectedCollection: string;
    isLoading: boolean;
    error: string | null;
    addCollection: (collection: Collection) => void;
    setCollections: (collections: Collection[]) => void;
    setSelectedCollection: (id: string) => void;
    setLoading: (loading: boolean) => void;
    setError: (error: string | null) => void;
}

export const useCollectionsStore = create<CollectionsState>()(
    persist(
        (set) => ({
            collections: [],
            selectedCollection: '',
            isLoading: false,
            error: null,
            addCollection: (collection) => 
                set((state) => ({ 
                    collections: [...state.collections, collection] 
                })),
            setCollections: (collections) => 
                set({ collections }),
            setSelectedCollection: (id) => 
                set({ selectedCollection: id }),
            setLoading: (loading) => 
                set({ isLoading: loading }),
            setError: (error) => 
                set({ error }),
        }),
        {
            name: 'collections-storage',
        }
    )
); 