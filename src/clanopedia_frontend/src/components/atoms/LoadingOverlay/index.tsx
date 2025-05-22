// src/components/atoms/LoadingOverlay/LoadingOverlay.tsx
import React from 'react';
import { LoadingSpinner } from '../LoadingSpinner/';
import './LoadingOverlay.scss';

interface LoadingOverlayProps {
    message?: string;
    transparent?: boolean;
    className?: string;
}

export const LoadingOverlay: React.FC<LoadingOverlayProps> = ({
    message = 'Loading...',
    transparent = false,
    className = ''
}) => {
    return (
        <div
            className={`loading-overlay ${transparent ? 'loading-overlay--transparent' : ''} ${className}`}
            role="dialog"
            aria-label="Loading"
            aria-live="polite"
        >
            <div className="loading-overlay__content">
                <LoadingSpinner size="large" color="primary" />
                {message && (
                    <p className="loading-overlay__message">
                        {message}
                    </p>
                )}
            </div>
        </div>
    );
};