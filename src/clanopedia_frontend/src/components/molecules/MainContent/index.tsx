// src/components/molecules/MainContent/MainContent.tsx
import React from 'react';
import { DocumentsTab } from './DocumentsTab';
import { SearchTab } from './SearchTab';
import { ProposalsTab } from './ProposalTab';
import { WelcomeScreen } from './WelcomeScreen';
import { Collection } from '../../../hooks/useCollections';
import './MainContent.scss';

// Types (should be moved to types file)
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

type TabType = 'documents' | 'search' | 'proposals';

interface MainContentProps {
    selectedCollection: string;
    collections: Collection[];
    documents: Document[];
    proposals: Proposal[];
    searchResults: SearchResult[];
    searchQuery: string;
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
    onAddDocument: () => void;
    onVoteProposal: (proposalId: string, inFavor: boolean) => Promise<void>;
    loading: boolean;
}

export const MainContent: React.FC<MainContentProps> = ({
    selectedCollection,
    collections,
    documents,
    proposals,
    searchResults,
    searchQuery,
    activeTab,
    onTabChange,
    onAddDocument,
    onVoteProposal,
    loading,
}) => {
    const selectedCollectionData = collections.find(c => c.id === selectedCollection);

    // Show welcome screen if no collection is selected
    if (!selectedCollection) {
        return (
            <main className="main-content">
                <WelcomeScreen />
            </main>
        );
    }

    return (
        <main className="main-content">
            {/* Tabs Navigation */}
            <div className="tabs">
                <button
                    className={`tab ${activeTab === 'documents' ? 'tab--active' : ''}`}
                    onClick={() => onTabChange('documents')}
                >
                    📄 Documents ({documents.length})
                </button>
                <button
                    className={`tab ${activeTab === 'search' ? 'tab--active' : ''}`}
                    onClick={() => onTabChange('search')}
                >
                    🔍 Search Results ({searchResults.length})
                </button>
                <button
                    className={`tab ${activeTab === 'proposals' ? 'tab--active' : ''}`}
                    onClick={() => onTabChange('proposals')}
                >
                    🗳️ Proposals ({proposals.length})
                </button>
            </div>

            {/* Tab Content */}
            <div className="tab-content">
                {activeTab === 'documents' && (
                    <DocumentsTab
                        collection={selectedCollectionData}
                        documents={documents}
                        onAddDocument={onAddDocument}
                        loading={loading}
                    />
                )}

                {activeTab === 'search' && (
                    <SearchTab
                        searchQuery={searchQuery}
                        searchResults={searchResults}
                        loading={loading}
                    />
                )}

                {activeTab === 'proposals' && (
                    <ProposalsTab
                        proposals={proposals}
                        onVoteProposal={onVoteProposal}
                        loading={loading}
                    />
                )}
            </div>
        </main>
    );
};