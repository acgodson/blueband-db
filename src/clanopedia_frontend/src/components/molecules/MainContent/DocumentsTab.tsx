// src/components/molecules/MainContent/DocumentsTab.tsx
import React from 'react';
import { Collection } from '../../../hooks/useCollections';

interface Document {
    id: string;
    title: string;
    content: string;
    isEmbedded: boolean;
    createdAt: Date;
}

interface DocumentsTabProps {
    collection: Collection | undefined;
    documents: Document[];
    onAddDocument: () => void;
    loading: boolean;
}

export const DocumentsTab: React.FC<DocumentsTabProps> = ({
    collection,
    documents,
    onAddDocument,
    loading
}) => {
    if (!collection) return null;

    return (
        <div className="documents-tab">
            <div className="documents-tab__header">
                <h2>{collection.name}</h2>
                <button
                    className="btn btn--primary"
                    onClick={onAddDocument}
                    disabled={!collection || loading}
                >
                    + Add Document
                </button>
            </div>

            {documents.length === 0 ? (
                <div className="empty-state">
                    <p>No documents in this collection yet.</p>
                    <button
                        className="btn btn--primary"
                        onClick={onAddDocument}
                        disabled={!collection || loading}
                    >
                        Add Your First Document
                    </button>
                </div>
            ) : (
                <div className="documents-grid">
                    {documents.map(doc => (
                        <div key={doc.id} className="document-card">
                            <h3>{doc.title}</h3>
                            <p className="document-card__meta">
                                {doc.isEmbedded ? '📚 Embedded' : '📄 Raw'}
                            </p>
                            <p className="document-card__date">
                                Added {doc.createdAt.toLocaleDateString()}
                            </p>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};