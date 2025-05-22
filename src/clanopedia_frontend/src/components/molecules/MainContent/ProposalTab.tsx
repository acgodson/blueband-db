// src/components/molecules/MainContent/ProposalsTab.tsx
import React from 'react';

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

interface ProposalsTabProps {
    proposals: Proposal[];
    onVoteProposal: (proposalId: string, inFavor: boolean) => Promise<void>;
    loading: boolean;
}

export const ProposalsTab: React.FC<ProposalsTabProps> = ({
    proposals,
    onVoteProposal,
    loading
}) => {
    const getStatusBadgeStyle = (status: string) => {
        const baseStyle = {
            fontSize: '0.75rem',
            fontWeight: '500' as const,
            padding: '0.25rem 0.5rem',
            borderRadius: '9999px',
            textTransform: 'uppercase' as const,
            letterSpacing: '0.05em'
        };

        switch (status.toLowerCase()) {
            case 'active':
                return { ...baseStyle, backgroundColor: '#dbeafe', color: '#1d4ed8' };
            case 'approved':
                return { ...baseStyle, backgroundColor: '#dcfce7', color: '#166534' };
            case 'rejected':
                return { ...baseStyle, backgroundColor: '#fee2e2', color: '#dc2626' };
            case 'executed':
                return { ...baseStyle, backgroundColor: '#f3e8ff', color: '#7c3aed' };
            default:
                return { ...baseStyle, backgroundColor: '#f3f4f6', color: '#374151' };
        }
    };

    const styles = {
        proposalsTab: {
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
        proposalsList: {
            display: 'flex',
            flexDirection: 'column' as const,
            gap: '1rem',
            flex: 1,
            overflow: 'auto'
        },
        proposalCard: {
            backgroundColor: 'white',
            border: '1px solid #e5e7eb',
            borderRadius: '0.5rem',
            padding: '1.5rem',
            boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.1)',
            transition: 'box-shadow 0.2s'
        },
        proposalHeader: {
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            marginBottom: '1rem'
        },
        proposalTitle: {
            fontSize: '1rem',
            fontWeight: '600',
            color: '#374151',
            margin: 0,
            lineHeight: '1.4',
            flex: 1,
            marginRight: '1rem'
        },
        proposalContent: {
            marginBottom: '1rem'
        },
        documentInfo: {
            fontSize: '0.875rem',
            color: '#6b7280',
            marginBottom: '0.75rem',
            margin: 0
        },
        voteStats: {
            display: 'flex',
            gap: '1rem',
            alignItems: 'center'
        },
        voteStat: {
            display: 'flex',
            alignItems: 'center',
            gap: '0.25rem',
            fontSize: '0.875rem',
            fontWeight: '500'
        },
        voteStatFor: {
            color: '#059669'
        },
        voteStatAgainst: {
            color: '#dc2626'
        },
        proposalActions: {
            display: 'flex',
            gap: '0.75rem',
            marginTop: '1rem'
        },
        voteButton: {
            border: 'none',
            borderRadius: '0.375rem',
            padding: '0.5rem 1rem',
            fontSize: '0.875rem',
            fontWeight: '500',
            cursor: 'pointer',
            transition: 'all 0.2s',
            disabled: {
                opacity: 0.5,
                cursor: 'not-allowed'
            }
        },
        voteForButton: {
            backgroundColor: '#059669',
            color: 'white',
            ':hover': {
                backgroundColor: '#047857'
            }
        },
        voteAgainstButton: {
            backgroundColor: '#dc2626',
            color: 'white',
            ':hover': {
                backgroundColor: '#b91c1c'
            }
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
        }
    };

    const handleVote = async (proposalId: string, inFavor: boolean) => {
        try {
            await onVoteProposal(proposalId, inFavor);
        } catch (error) {
            console.error('Error voting on proposal:', error);
        }
    };

    return (
        <div style={styles.proposalsTab}>
            <h3 style={styles.title}>Active Proposals</h3>

            {proposals.length === 0 ? (
                <div style={styles.emptyState}>
                    <div style={styles.emptyStateIcon}>🗳️</div>
                    <p style={styles.emptyStateText}>No proposals yet</p>
                    <p style={styles.emptyStateSubtext}>
                        Proposals will appear here when they are created
                    </p>
                </div>
            ) : (
                <div style={styles.proposalsList}>
                    {proposals.map(proposal => (
                        <div
                            key={proposal.id}
                            style={styles.proposalCard}
                            onMouseEnter={(e) => {
                                (e.currentTarget as HTMLElement).style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1)';
                            }}
                            onMouseLeave={(e) => {
                                (e.currentTarget as HTMLElement).style.boxShadow = '0 1px 3px 0 rgba(0, 0, 0, 0.1)';
                            }}
                        >
                            <div style={styles.proposalHeader}>
                                <h4 style={styles.proposalTitle}>{proposal.description}</h4>
                                <span style={getStatusBadgeStyle(proposal.status)}>
                                    {proposal.status}
                                </span>
                            </div>

                            <div style={styles.proposalContent}>
                                <p style={styles.documentInfo}>
                                    Document: {proposal.documentTitle}
                                </p>
                                <div style={styles.voteStats}>
                                    <span style={{ ...styles.voteStat, ...styles.voteStatFor }}>
                                        👍 {proposal.votesFor}
                                    </span>
                                    <span style={{ ...styles.voteStat, ...styles.voteStatAgainst }}>
                                        👎 {proposal.votesAgainst}
                                    </span>
                                </div>
                            </div>

                            {proposal.status === 'pending' && (
                                <div style={styles.proposalActions}>
                                    <button
                                        style={{ ...styles.voteButton, ...styles.voteForButton }}
                                        onClick={() => handleVote(proposal.id, true)}
                                        disabled={loading}
                                        onMouseEnter={(e) => {
                                            if (!loading) {
                                                (e.target as HTMLButtonElement).style.backgroundColor = '#047857';
                                            }
                                        }}
                                        onMouseLeave={(e) => {
                                            if (!loading) {
                                                (e.target as HTMLButtonElement).style.backgroundColor = '#059669';
                                            }
                                        }}
                                    >
                                        {loading ? 'Voting...' : 'Vote For'}
                                    </button>
                                    <button
                                        style={{ ...styles.voteButton, ...styles.voteAgainstButton }}
                                        onClick={() => handleVote(proposal.id, false)}
                                        disabled={loading}
                                        onMouseEnter={(e) => {
                                            if (!loading) {
                                                (e.target as HTMLButtonElement).style.backgroundColor = '#b91c1c';
                                            }
                                        }}
                                        onMouseLeave={(e) => {
                                            if (!loading) {
                                                (e.target as HTMLButtonElement).style.backgroundColor = '#dc2626';
                                            }
                                        }}
                                    >
                                        {loading ? 'Voting...' : 'Vote Against'}
                                    </button>
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};