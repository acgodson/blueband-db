// src/components/molecules/Navbar/Navbar.tsx
import React, { useEffect, useState } from 'react';
import { useAuthContext } from '../../../providers/AuthProvider';
import { useAuth } from '../../../hooks/useAuth';
import './Navbar.scss';

interface NavbarProps {
    onClaimTokens?: () => void;
    userTokens?: number;
}

export const Navbar: React.FC<NavbarProps> = ({
    onClaimTokens,
    userTokens
}) => {
    const { isAuthenticated, principal, login, logout } = useAuthContext();
    const { tokenActor } = useAuth();
    const [balance, setBalance] = useState<number>(0);
    const [isMinting, setIsMinting] = useState(false);
    const [showTokenDetails, setShowTokenDetails] = useState(false);
    const [tokenDetails, setTokenDetails] = useState<{
        name: string;
        symbol: string;
        decimals: number;
    } | null>(null);

    useEffect(() => {
        const fetchBalance = async () => {
            if (isAuthenticated && principal && tokenActor) {
                try {
                    const balance = await tokenActor.icrc1_balance_of({
                        owner: principal,
                        subaccount: []
                    });
                    setBalance(Number(balance) / 1e8); // Convert from atomic units
                } catch (error) {
                    console.error('Error fetching balance:', error);
                }
            }
        };

        const fetchTokenDetails = async () => {
            if (tokenActor) {
                try {
                    const [name, symbol, decimals] = await Promise.all([
                        tokenActor.icrc1_name(),
                        tokenActor.icrc1_symbol(),
                        tokenActor.icrc1_decimals()
                    ]);
                    setTokenDetails({
                        name,
                        symbol,
                        decimals: Number(decimals)
                    });
                } catch (error) {
                    console.error('Error fetching token details:', error);
                }
            }
        };

        fetchBalance();
        fetchTokenDetails();

        // Set up polling for balance updates
        const interval = setInterval(fetchBalance, 10000); // Poll every 10 seconds
        return () => clearInterval(interval);
    }, [isAuthenticated, principal, tokenActor]);

    const handleMint = async () => {
        if (!tokenActor || !principal || isMinting) return;

        setIsMinting(true);
        try {
            await tokenActor.mint(principal, BigInt(100 * 1e8)); // Mint 100 tokens
            // Balance will be updated by the polling effect
        } catch (error) {
            console.error('Error minting tokens:', error);
        } finally {
            setIsMinting(false);
        }
    };

    const handleAuth = () => {
        if (isAuthenticated) {
            logout();
        } else {
            login();
        }
    };

    return (
        <header className="navbar">
            <div className="navbar__brand">
                <h1>🌟 Clanopedia</h1>
                <span className="navbar__subtitle">Powered by Blueband Vector DB</span>
            </div>

            <div className="navbar__actions">
                {isAuthenticated ? (
                    <>
                        <div className="user-info">
                            <span className="user-info__label">Principal:</span>
                            <span
                                className="user-info__value"
                                title={principal?.toString()}
                            >
                                {principal?.toString().slice(0, 8)}...
                            </span>
                        </div>

                        <div 
                            className="token-display"
                            onMouseEnter={() => setShowTokenDetails(true)}
                            onMouseLeave={() => setShowTokenDetails(false)}
                        >
                            <span className="token-display__label">Tokens:</span>
                            <span className="token-display__value">{balance.toFixed(2)}</span>
                            {showTokenDetails && tokenDetails && (
                                <div className="token-details">
                                    <h4>{tokenDetails.name}</h4>
                                    <p>Symbol: {tokenDetails.symbol}</p>
                                    <p>Decimals: {tokenDetails.decimals}</p>
                                </div>
                            )}
                        </div>

                        <button
                            className="btn btn--outline btn--sm"
                            onClick={handleMint}
                            disabled={isMinting}
                        >
                            {isMinting ? 'Minting...' : '🎁 Mint Tokens'}
                        </button>

                        <button
                            className="btn btn--outline btn--sm"
                            onClick={handleAuth}
                        >
                            Sign Out
                        </button>

                        <div className="status status--connected">
                            🟢 Connected
                        </div>
                    </>
                ) : (
                    <>
                        <button
                            className="btn btn--primary"
                            onClick={handleAuth}
                        >
                            🔐 Sign In
                        </button>

                        <div className="status status--disconnected">
                            🔴 Disconnected
                        </div>
                    </>
                )}
            </div>
        </header>
    );
};