// src/components/molecules/MainContent/WelcomeScreen.tsx
import React from 'react';

export const WelcomeScreen: React.FC = () => {
    const styles = {
        welcomeScreen: {
            display: 'flex',
            flexDirection: 'column' as const,
            alignItems: 'center',
            justifyContent: 'center',
            height: '100%',
            textAlign: 'center' as const,
            padding: '2rem',
            color: '#6b7280'
        },
        title: {
            fontSize: '2rem',
            fontWeight: '600',
            marginBottom: '1rem',
            color: '#374151'
        },
        description: {
            fontSize: '1.125rem',
            lineHeight: '1.6',
            maxWidth: '600px'
        }
    };

    return (
        <div style={styles.welcomeScreen}>
            <h2 style={styles.title}>Welcome to Blueband Vector Database</h2>
            <p style={styles.description}>
                Select a collection from the sidebar or create a new one to get started.
            </p>
        </div>
    );
};