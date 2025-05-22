// src/components/molecules/Sidebar/Sidebar.tsx
import React from 'react';
import './Sidebar.scss';

interface Collection {
  id: string;
  name: string;
  documentCount: number;
  createdAt: Date;
}

interface SidebarProps {
  collections: Collection[];
  selectedCollection: string;
  onSelectCollection: (collectionId: string) => void;
  onCreateCollection: () => void;
  searchQuery: string;
  onSearchQueryChange: (query: string) => void;
  onSearch: () => void;
  canSearch: boolean;
  loading: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  collections,
  selectedCollection,
  onSelectCollection,
  onCreateCollection,
  searchQuery,
  onSearchQueryChange,
  onSearch,
  canSearch,
  loading
}) => {
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && canSearch) {
      onSearch();
    }
  };

  return (
    <aside className="sidebar">
      <div className="sidebar__header">
        <h2>Collections</h2>
        <button
          className="btn btn--primary btn--sm"
          onClick={onCreateCollection}
          disabled={loading}
        >
          + New
        </button>
      </div>

      <div className="collections-list">
        {collections.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state__icon">📁</div>
            <p className="empty-state__title">No collections yet</p>
            <p className="empty-state__description">
              Create your first knowledge collection to get started
            </p>
            <button
              className="btn btn--primary"
              onClick={onCreateCollection}
              disabled={loading}
            >
              Create Collection
            </button>
          </div>
        ) : (
          <>
            {collections.map(collection => (
              <div
                key={collection.id}
                className={`collection-item ${
                  selectedCollection === collection.id ? 'collection-item--active' : ''
                }`}
                onClick={() => onSelectCollection(collection.id)}
                role="button"
                tabIndex={0}
                onKeyPress={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    onSelectCollection(collection.id);
                  }
                }}
              >
                <div className="collection-item__content">
                  <div className="collection-item__name">{collection.name}</div>
                  <div className="collection-item__meta">
                    <span className="collection-item__count">
                      {collection.documentCount} docs
                    </span>
                    <span className="collection-item__date">
                      {new Date(collection.createdAt).toLocaleDateString()}
                    </span>
                  </div>
                </div>
                <div className="collection-item__icon">
                  📚
                </div>
              </div>
            ))}
          </>
        )}
      </div>

      {/* Search Section */}
      <div className="search-section">
        <h3>🔍 Semantic Search</h3>
        <div className="search-input-group">
          <input
            type="text"
            placeholder="Search knowledge base..."
            value={searchQuery}
            onChange={(e) => onSearchQueryChange(e.target.value)}
            onKeyPress={handleKeyPress}
            className="search-input"
            disabled={loading}
          />
          <button
            className="btn btn--primary btn--sm"
            onClick={onSearch}
            disabled={!canSearch || loading}
            title={!selectedCollection ? 'Select a collection first' : 'Search documents'}
          >
            {loading ? '⏳' : '🔍'}
          </button>
        </div>
        {!selectedCollection && searchQuery && (
          <p className="search-hint">
            💡 Select a collection to search
          </p>
        )}
      </div>
    </aside>
  );
};