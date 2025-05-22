// src/providers/AuthProvider.tsx
import React, { createContext, useContext, ReactNode } from 'react';
import { useAuth } from '../hooks/useAuth';

interface AuthContextType {
    isAuthenticated: boolean;
    identity: any;
    principal: any;
    login: () => void;
    logout: () => void;
    bluebandActor: any;
    tokenActor: any;
    loading?: boolean;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const useAuthContext = () => {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error('useAuthContext must be used within an AuthProvider');
    }
    return context;
};

interface AuthProviderProps {
    children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
    const auth = useAuth();

    return (
        <AuthContext.Provider value={auth}>
            {children}
        </AuthContext.Provider>
    );
};