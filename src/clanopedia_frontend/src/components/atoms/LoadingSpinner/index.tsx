// src/components/atoms/LoadingSpinner/index.tsx
import React from 'react';
import './LoadingSpinner.scss';

interface LoadingSpinnerProps {
    size?: 'small' | 'medium' | 'large';
    color?: 'primary' | 'white' | 'gray';
    className?: string;
}

export const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
    size = 'medium',
    color = 'primary',
    className = ''
}) => {
    return (
        <div
            className={`loading-spinner loading-spinner--${size} loading-spinner--${color} ${className}`}
            role="status"
            aria-label="Loading"
        >
            <span className="loading-spinner__sr-only">Loading...</span>
        </div>
    );
};