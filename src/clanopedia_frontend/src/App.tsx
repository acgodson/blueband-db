import React, { useState } from 'react';
import { AuthProvider } from './providers/AuthProvider';
import { useAuthContext } from './providers/AuthProvider';
import { LoginScreen, Dashboard, Navbar } from './components';
import './index.scss';


const AppContent: React.FC = () => {
  const { isAuthenticated } = useAuthContext();
  const [userTokens, setUserTokens] = useState(100); // Mock tokens

  const handleClaimTokens = () => {
    setUserTokens(prev => prev + 50);
    // Add actual token claiming logic here
  };

  // Show login screen if not authenticated
  if (!isAuthenticated) {
    return <LoginScreen />;
  }

  // Show main dashboard if authenticated
  return (
    <div className="app">
      <Navbar
        userTokens={userTokens}
        onClaimTokens={handleClaimTokens}
      />
      <Dashboard />
    </div>
  );
};

// Main App with Auth Provider
function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}

export default App;