// src/components/molecules/MainContent/SearchTab.tsx
import React from 'react';

interface SearchResult {
    id: string;
    title: string;
    content: string;
    score: number;
}

interface SearchTabProps {
    searchQuery: string;
    searchResults: SearchResult[];
    loading: boolean;
}

export const SearchTab: React.FC<SearchTabProps> = ({
    searchQuery,
    searchResults,
    loading
}) => {
    const styles = {
        searchTab: {
            height: '100%',
            display: 'flex',
            flexDirection: 'column' as const
        },
        title: {
            fontSize: '1.25rem',
            fontWeight: '600',
            color: '#374151',
            marginBottom: '1.5rem',
            paddingBottom: '1rem',
            borderBottom: '1px solid #e5e7eb'
        },
        searchResults: {
            display: 'flex',
            flexDirection: 'column' as const,
            gap: '1rem',
            flex: 1,
            overflow: 'auto'
        },
        searchResult: {
            backgroundColor: 'white',
            border: '1px solid #e5e7eb',
            borderRadius: '0.5rem',
            padding: '1rem',
            boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.1)',
            transition: 'box-shadow 0.2s, transform 0.2s',
            cursor: 'pointer'
        },
        resultHeader: {
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            marginBottom: '0.75rem'
        },
        resultTitle: {
            fontSize: '1rem',
            fontWeight: '600',
            color: '#374151',
            margin: 0,
            lineHeight: '1.4'
        },
        scoreBadge: {
            fontSize: '0.75rem',
            fontWeight: '500',
            padding: '0.25rem 0.5rem',
            borderRadius: '9999px',
            backgroundColor: '#dbeafe',
            color: '#1d4ed8'
        },
        resultText: {
            fontSize: '0.875rem',
            color: '#6b7280',
            lineHeight: '1.6',
            margin: 0
        },
        emptyState: {
            display: 'flex',
            flexDirection: 'column' as const,
            alignItems: 'center',
            justifyContent: 'center',
            height: '300px',
            color: '#6b7280',
            textAlign: 'center' as const
        },
        emptyStateIcon: {
            fontSize: '3rem',
            marginBottom: '1rem'
        },
        emptyStateText: {
            fontSize: '1rem',
            marginBottom: '0.5rem'
        },
        emptyStateSubtext: {
            fontSize: '0.875rem',
            color: '#9ca3af'
        },
        loadingState: {
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            height: '200px',
            color: '#6b7280'
        },
        spinner: {
            width: '2rem',
            height: '2rem',
            border: '2px solid #e5e7eb',
            borderTop: '2px solid #3b82f6',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite'
        }
    };

    if (loading) {
        return (
            <div style={styles.searchTab}>
                <h3 style={styles.title}>Searching...</h3>
                <div style={styles.loadingState}>
                    <div style={styles.spinner}></div>
                </div>
            </div>
        );
    }

    return (
        <div style={styles.searchTab}>
            <h3 style={styles.title}>
                Search Results for "{searchQuery}"
            </h3>

            {searchResults.length === 0 ? (
                <div style={styles.emptyState}>
                    <div style={styles.emptyStateIcon}>🔍</div>
                    <p style={styles.emptyStateText}>No search results found</p>
                    <p style={styles.emptyStateSubtext}>
                        Try different keywords or check your spelling
                    </p>
                </div>
            ) : (
                <div style={styles.searchResults}>
                    {searchResults.map((result, index) => (
                        <div
                            key={`${result.id}-${index}`}
                            style={styles.searchResult}
                            onMouseEnter={(e) => {
                                (e.currentTarget as HTMLElement).style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1)';
                                (e.currentTarget as HTMLElement).style.transform = 'translateY(-1px)';
                            }}
                            onMouseLeave={(e) => {
                                (e.currentTarget as HTMLElement).style.boxShadow = '0 1px 3px 0 rgba(0, 0, 0, 0.1)';
                                (e.currentTarget as HTMLElement).style.transform = 'translateY(0)';
                            }}
                        >
                            <div style={styles.resultHeader}>
                                <h4 style={styles.resultTitle}>
                                    {result.title}
                                </h4>
                                <span style={styles.scoreBadge}>
                                    {(result.score * 100).toFixed(1)}% match
                                </span>
                            </div>
                            <p style={styles.resultText}>
                                {result.content}
                            </p>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};