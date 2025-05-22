// src/components/organisms/LoginScreen/LoginScreen.tsx
import React from 'react';
import { useAuthContext } from '../../../providers/AuthProvider';
import './LoginScreen.scss';

export const LoginScreen: React.FC = () => {
    const { login } = useAuthContext();

    return (
        <div className="login-screen">
            <div className="login-container">
                <h1>🌟 Welcome to Clanopedia</h1>
                <p className="login-subtitle">
                    Decentralized Knowledge Platform powered by Blueband Vector Database
                </p>

                <div className="login-card">
                    <h2>Sign in to continue</h2>
                    <p>
                        Connect with Internet Identity to access your collections,
                        create knowledge bases, and participate in governance.
                    </p>

                    <button
                        className="btn btn--primary btn--large"
                        onClick={login}
                    >
                        🔐 Sign in with Internet Identity
                    </button>
                </div>

                <div className="login-features">
                    <h3>What you can do with Clanopedia:</h3>
                    <ul>
                        <li>📚 Create and organize knowledge collections</li>
                        <li>🔍 Perform intelligent semantic search across documents</li>
                        <li>🗳️ Participate in decentralized governance (DAO)</li>
                        <li>🎁 Earn tokens for community contributions</li>
                        <li>🤝 Collaborate with communities worldwide</li>
                        <li>🔒 Own your data with blockchain security</li>
                    </ul>
                </div>

                <div className="login-footer">
                    <p>
                        <small>
                            Powered by Internet Computer Protocol & Blueband Vector Database
                        </small>
                    </p>
                </div>
            </div>
        </div>
    );
};