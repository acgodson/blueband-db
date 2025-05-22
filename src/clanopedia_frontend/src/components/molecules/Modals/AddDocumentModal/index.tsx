// src/components/molecules/Modals/AddDocumentModal/AddDocumentModal.tsx
import React, { useState } from 'react';
import { LoadingSpinner } from '../../../atoms/LoadingSpinner/';
import { DocumentProcessor } from '../../../../utils/documentProcessor';
import './AddDocumentModal.scss';

interface AddDocumentModalProps {
    onClose: () => void;
    onSubmit: (title: string, content: string) => Promise<void>;
    loading?: boolean;
    collectionId: string;
}

export const AddDocumentModal: React.FC<AddDocumentModalProps> = ({
    onClose,
    onSubmit,
    loading = false,
    collectionId
}) => {
    const [title, setTitle] = useState('');
    const [content, setContent] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!title.trim() || !content.trim()) {
            setError('Both title and content are required');
            return;
        }

        try {
            setError('');
            // Process the document
            const processedDoc = DocumentProcessor.processDocument(content, title);
            const operations = DocumentProcessor.prepareForUpload(processedDoc, collectionId);
            
            // Submit the document
            await onSubmit(operations.addDocument.title, operations.addDocument.content);
            // Modal will be closed by parent component on success
        } catch (err) {
            setError('Failed to add document. Please try again.');
            console.error('Error adding document:', err);
        }
    };

    const handleOverlayClick = (e: React.MouseEvent) => {
        if (e.target === e.currentTarget && !loading) {
            onClose();
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Escape') {
            onClose();
        }
    };

    return (
        <div
            className="modal-overlay"
            onClick={handleOverlayClick}
            onKeyDown={handleKeyDown}
            role="dialog"
            aria-modal="true"
            aria-labelledby="add-document-title"
        >
            <div className="modal add-document-modal" onClick={e => e.stopPropagation()}>
                <form onSubmit={handleSubmit}>
                    <div className="modal__header">
                        <h3 id="add-document-title">Add New Document</h3>
                        <button
                            type="button"
                            className="modal__close"
                            onClick={onClose}
                            aria-label="Close modal"
                            disabled={loading}
                        >
                            ×
                        </button>
                    </div>

                    <div className="modal__content">
                        {error && (
                            <div className="error-message" role="alert">
                                {error}
                            </div>
                        )}

                        <div className="form-group">
                            <label htmlFor="document-title" className="form-label">
                                Document Title *
                            </label>
                            <input
                                id="document-title"
                                type="text"
                                value={title}
                                onChange={(e) => setTitle(e.target.value)}
                                placeholder="e.g., AI Research Paper, Community Guidelines..."
                                className={`form-input ${error ? 'input--error' : ''}`}
                                disabled={loading}
                                maxLength={200}
                                required
                                autoFocus
                            />
                            <div className="form-hint">
                                Give your document a clear, descriptive title
                            </div>
                        </div>

                        <div className="form-group">
                            <label htmlFor="document-content" className="form-label">
                                Document Content *
                            </label>
                            <textarea
                                id="document-content"
                                value={content}
                                onChange={(e) => setContent(e.target.value)}
                                placeholder="Paste your document content here. This can be articles, research papers, guides, or any text-based content you want to make searchable..."
                                className={`form-textarea ${error ? 'textarea--error' : ''}`}
                                rows={12}
                                disabled={loading}
                                required
                            />
                            <div className="form-hint">
                                {content.length > 0 && (
                                    <span className="character-count">
                                        {content.length.toLocaleString()} characters
                                    </span>
                                )}
                                <span>Rich text content will be processed for semantic search</span>
                            </div>
                        </div>

                        <div className="form-info">
                            <div className="info-card">
                                <div className="info-card__icon">💡</div>
                                <div className="info-card__content">
                                    <h4>How document processing works:</h4>
                                    <ul>
                                        <li>Your document will be split into semantic chunks</li>
                                        <li>AI embeddings will be generated for intelligent search</li>
                                        <li>Content becomes searchable across your collection</li>
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="modal__footer">
                        <button
                            type="button"
                            className="btn btn--outline"
                            onClick={onClose}
                            disabled={loading}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="btn btn--primary"
                            disabled={!title.trim() || !content.trim() || loading}
                        >
                            {loading ? (
                                <>
                                    <LoadingSpinner size="small" color="white" />
                                    Adding Document...
                                </>
                            ) : (
                                <>
                                    📄 Add Document
                                </>
                            )}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};