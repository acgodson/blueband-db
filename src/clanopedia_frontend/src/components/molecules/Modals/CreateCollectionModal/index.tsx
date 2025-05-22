// src/components/molecules/Modals/CreateCollectionModal/index.tsx
import React, { useState } from 'react';
import { useAuthContext } from '../../../../providers/AuthProvider';
import { useCollectionsStore } from '../../../../store/collectionsStore';
import './CreateCollectionModal.scss';

interface CreateCollectionModalProps {
    onClose: () => void;
    onSubmit: (name: string) => Promise<void>;
}

export const CreateCollectionModal: React.FC<CreateCollectionModalProps> = ({ onClose, onSubmit }) => {
    const { bluebandActor } = useAuthContext();
    const [name, setName] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);
    
    const { addCollection, setSelectedCollection } = useCollectionsStore();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim()) return;

        try {
            setLoading(true);
            setError(null);
            setSuccess(null);

            await onSubmit(name);
            setSuccess('Collection created successfully! You can now close this dialog.');
        } catch (err) {
            console.error('Error creating collection:', err);
            setError(err instanceof Error ? err.message : 'An error occurred while creating the collection');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="modal-overlay">
            <div className="modal">
                <div className="modal__header">
                    <h3>Create New Collection</h3>
                </div>
                <div className="modal__content">
                    <form onSubmit={handleSubmit}>
                        <div className="form-group">
                            <label htmlFor="collectionName">Collection Name</label>
                            <input
                                type="text"
                                id="collectionName"
                                className="input"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="Enter collection name"
                                disabled={loading}
                                required
                            />
                        </div>

                        {error && <div className="form-error">{error}</div>}
                        {success && <div className="form-error" style={{ color: 'green' }}>{success}</div>}

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
                                disabled={loading || !name.trim()}
                            >
                                {loading ? 'Creating...' : 'Create Collection'}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    );
};