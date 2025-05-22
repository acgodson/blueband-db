// src/components/organisms/Dashboard/index.tsx
import React, { useState, useEffect } from 'react';
import { useAuthContext } from '../../../providers/AuthProvider';
import { Sidebar } from '../../molecules/Sidebar';
import { MainContent } from '../../molecules/MainContent';
import { CreateCollectionModal } from '../../molecules/Modals/CreateCollectionModal';
import { AddDocumentModal } from '../../molecules/Modals/AddDocumentModal';
import { LoadingOverlay } from '../../atoms/LoadingOverlay/';
import { useCollectionsStore } from '../../../store/collectionsStore';
import './Dashboard.scss';

// Types (move these to a separate types file later)
interface Document {
    id: string;
    title: string;
    content: string;
    isEmbedded: boolean;
    createdAt: Date;
}

interface Proposal {
    id: string;
    documentId: string;
    documentTitle: string;
    description: string;
    status: 'pending' | 'approved' | 'rejected';
    votesFor: number;
    votesAgainst: number;
    createdAt: Date;
}

interface SearchResult {
    id: string;
    title: string;
    content: string;
    score: number;
}

interface Collection {
    id: string;
    name: string;
    bucketId: string;
    createdAt: Date;
    documentCount: number;
}

export const Dashboard: React.FC = () => {
    const { bluebandActor } = useAuthContext();
    const {
        collections,
        selectedCollection,
        setSelectedCollection,
        addCollection,
        setCollections,
        isLoading: collectionsLoading,
        setLoading: setCollectionsLoading,
        error: collectionsError,
        setError: setCollectionsError
    } = useCollectionsStore();

    // Core state
    const [documents, setDocuments] = useState<Document[]>([]);
    const [proposals, setProposals] = useState<Proposal[]>([]);
    const [searchResults, setSearchResults] = useState<SearchResult[]>([]);

    // UI state
    const [loading, setLoading] = useState(false);
    const [activeTab, setActiveTab] = useState<'documents' | 'search' | 'proposals'>('documents');
    const [showCreateCollection, setShowCreateCollection] = useState(false);
    const [showAddDocument, setShowAddDocument] = useState(false);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    // Form state
    const [searchQuery, setSearchQuery] = useState('');

    // Load collections on mount
    useEffect(() => {
        loadCollections();
    }, [bluebandActor]);

    // Load documents for selected collection
    useEffect(() => {
        if (selectedCollection && bluebandActor) {
            loadDocuments(selectedCollection);
            loadProposals(selectedCollection);
        }
    }, [selectedCollection, bluebandActor]);

    // Clear success message after 3 seconds
    useEffect(() => {
        if (successMessage) {
            const timer = setTimeout(() => {
                setSuccessMessage(null);
            }, 3000);
            return () => clearTimeout(timer);
        }
    }, [successMessage]);

    const loadCollections = async () => {
        if (!bluebandActor) return;

        try {
            setCollectionsLoading(true);
            setCollectionsError(null);

            // Get all collections from the contract
            const result = await bluebandActor.listCollections();

            if (result) {
                const collectionsList = result.map((col: any) => ({
                    id: col.id,
                    name: col.name,
                    bucketId: col.bucketId,
                    createdAt: new Date(Number(col.createdAt) / 1000000),
                    documentCount: 0 // We'll update this when loading documents
                }));

                setCollections(collectionsList);
            }
        } catch (error) {
            console.error('Error loading collections:', error);
            setCollectionsError('Failed to load collections');
        } finally {
            setCollectionsLoading(false);
        }
    };

    const loadDocuments = async (collectionId: string) => {
        try {
            setLoading(true);
            const result = await bluebandActor.getDocumentList(collectionId);

            if (result && result.length > 0) {
                const docs: Document[] = result[0]?.map((doc: any) => ({
                    id: doc.id,
                    title: doc.title,
                    content: '',
                    isEmbedded: doc.is_embedded,
                    createdAt: new Date(Number(doc.timestamp) / 1000000)
                })) || [];
                setDocuments(docs);

                // Update collection's document count
                setCollections(collections.map(col =>
                    col.id === collectionId
                        ? { ...col, documentCount: docs.length }
                        : col
                ));
            } else {
                setDocuments([]);
                // Update collection's document count to 0
                setCollections(collections.map(col =>
                    col.id === collectionId
                        ? { ...col, documentCount: 0 }
                        : col
                ));
            }
        } catch (error) {
            console.error('Error loading documents:', error);
            setDocuments([]);
        } finally {
            setLoading(false);
        }
    };

    const loadProposals = async (collectionId: string) => {
        try {
            const result = await bluebandActor.listProposals(collectionId, []);

            const props: Proposal[] = result.map((prop: any) => ({
                id: prop.id,
                documentId: prop.documentId,
                documentTitle: `Document ${prop.documentId}`,
                description: prop.description?.[0] || 'No description',
                status: Object.keys(prop.status)[0] as any,
                votesFor: Number(prop.votesFor),
                votesAgainst: Number(prop.votesAgainst),
                createdAt: new Date(Number(prop.created) / 1000000)
            }));
            setProposals(props);
        } catch (error) {
            console.error('Error loading proposals:', error);
            setProposals([]);
        }
    };

    const handleCreateCollection = async (name: string) => {
        try {
            setLoading(true);
            const collectionId = name.toLowerCase().replace(/\s+/g, '-');
            
            // Call backend to create collection
            const result = await bluebandActor.createCollection(
                collectionId,
                [], // token_canister - not using tokens for now
                [], // voting_threshold - will use default (51%)
                [], // quorum_threshold - will use default (10%)
                []  // expiry_time - will use default (7 days)
            );

            if (!result) {
                throw new Error('Failed to create collection');
            }

            const collection: Collection = {
                id: collectionId,
                name: name,
                bucketId: result.toString(), // Store the bucket principal
                createdAt: new Date(),
                documentCount: 0
            };
            
            addCollection(collection);
            setSuccessMessage('Collection created successfully!');
            setShowCreateCollection(false);
            setSelectedCollection(collectionId);
        } catch (error) {
            console.error('Error creating collection:', error);
            throw error;
        } finally {
            setLoading(false);
        }
    };

    const addDocument = async (title: string, content: string) => {
        if (!selectedCollection || !bluebandActor) return;

        try {
            setLoading(true);
            setSuccessMessage('Adding document...');

            console.log('Adding document to collection:', selectedCollection);
            console.log('Document title:', title);
            console.log('Content length:', content.length);

            // Check if user is admin
            const isAdmin = await bluebandActor.isCollectionAdmin(selectedCollection);
            console.log('Is user admin:', isAdmin);

            if (!isAdmin) {
                throw new Error('You must be an admin to add documents to this collection');
            }

            // First add the document
            const result = await bluebandActor.addDocument(
                selectedCollection,
                title,
                content
            );

            console.log('Add document result:', result);

            if (result && result.length > 0) {
                setSuccessMessage('Document added! Creating embedding proposal...');

                console.log('Creating embedding proposal for document:', result[0].documentId?.[0] || '');

                // Then create an embedding proposal
                const proposalResult = await bluebandActor.createEmbeddingProposal(
                    selectedCollection,
                    result[0].documentId?.[0] || '',
                    { Some: `Embed document: ${title}` }
                );

                console.log('Embedding proposal result:', proposalResult);

                if (proposalResult) {
                    setSuccessMessage('Document added and embedding proposal created!');
                    await loadDocuments(selectedCollection);
                    await loadProposals(selectedCollection);
                    setShowAddDocument(false);
                } else {
                    setSuccessMessage('Document added but embedding proposal failed. You can try embedding later.');
                    await loadDocuments(selectedCollection);
                    setShowAddDocument(false);
                }
            } else {
                console.error('Add document failed - no result returned');
                throw new Error('Failed to add document - no result returned from backend');
            }
        } catch (error) {
            console.error('Error adding document:', error);
            if (error instanceof Error) {
                console.error('Error details:', {
                    message: error.message,
                    stack: error.stack,
                    name: error.name
                });
            }
            setSuccessMessage('Failed to add document. Please try again.');
            throw error;
        } finally {
            setLoading(false);
        }
    };

    const searchDocuments = async (query: string) => {
        if (!selectedCollection || !query.trim()) return;

        try {
            setLoading(true);
            const result = await bluebandActor.searchDocuments(
                selectedCollection,
                query
            );

            if (result && result.length > 0) {
                const results: SearchResult[] = result[0]?.map((doc: any) => ({
                    id: doc.id,
                    title: doc.title,
                    content: doc.content,
                    score: Number(doc.score)
                })) || [];
                setSearchResults(results);
                setActiveTab('search');
            } else {
                setSearchResults([]);
            }
        } catch (error) {
            console.error('Error searching documents:', error);
            setSearchResults([]);
        } finally {
            setLoading(false);
        }
    };

    const voteOnProposal = async (proposalId: string, inFavor: boolean) => {
        if (!selectedCollection) return;

        try {
            setLoading(true);
            await bluebandActor.voteOnProposal(
                selectedCollection,
                proposalId,
                inFavor
            );
            await loadProposals(selectedCollection);
        } catch (error) {
            console.error('Error voting on proposal:', error);
            throw error;
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="dashboard">
            {successMessage && (
                <div className="success-message">
                    {successMessage}
                </div>
            )}
            <div className="dashboard__layout">
                <Sidebar
                    collections={collections}
                    selectedCollection={selectedCollection}
                    onSelectCollection={setSelectedCollection}
                    onCreateCollection={() => setShowCreateCollection(true)}
                    searchQuery={searchQuery}
                    onSearchQueryChange={setSearchQuery}
                    onSearch={() => searchDocuments(searchQuery)}
                    canSearch={!!selectedCollection && !!searchQuery.trim()}
                    loading={loading || collectionsLoading}
                />

                <MainContent
                    selectedCollection={selectedCollection}
                    collections={collections}
                    documents={documents}
                    proposals={proposals}
                    searchResults={searchResults}
                    searchQuery={searchQuery}
                    activeTab={activeTab}
                    onTabChange={setActiveTab}
                    onAddDocument={() => setShowAddDocument(true)}
                    onVoteProposal={voteOnProposal}
                    loading={loading}
                />
            </div>

            {/* Modals */}
            {showCreateCollection && (
                <CreateCollectionModal
                    onClose={() => setShowCreateCollection(false)}
                    onSubmit={handleCreateCollection}
                />
            )}

            {showAddDocument && selectedCollection && (
                <AddDocumentModal
                    onClose={() => setShowAddDocument(false)}
                    onSubmit={addDocument}
                    loading={loading}
                    collectionId={selectedCollection}
                />
            )}

            {loading && <LoadingOverlay />}
        </div>
    );
};