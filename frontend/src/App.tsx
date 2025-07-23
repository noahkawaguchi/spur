import { BrowserRouter, Route, Routes } from 'react-router-dom';
import LoginPage from './pages/LoginPage';
import ProfilePage from './pages/ProfilePage';
import FriendsPage from './pages/FriendsPage';
import './App.css';
import HomePage from './pages/HomePage';
import NotFoundPage from './pages/NotFoundPage';
import MainLayout from './layouts/MainLayout';
import FriendContentPage from './pages/FriendContentPage';
import { ErrorBoundary } from 'react-error-boundary';
import ErrorFallback from './components/ErrorFallback';
import FeedPage from './pages/FeedPage';
import CreatePage from './pages/CreatePage';
import FriendRequestsPage from './pages/FriendRequestsPage';

const App = () => {
  return (
    <BrowserRouter>
      <ErrorBoundary FallbackComponent={ErrorFallback}>
        <Routes>
          <Route path='/login' element={<LoginPage />} />
          <Route element={<MainLayout />}>
            <Route path='/' element={<HomePage />} />
            <Route path='/feed' element={<FeedPage />} />
            <Route path='/create' element={<CreatePage />} />
            <Route path='/friends' element={<FriendsPage />} />
            <Route path='/friends/:username' element={<FriendContentPage />} />
            <Route path='/requests' element={<FriendRequestsPage />} />
            <Route path='/profile' element={<ProfilePage />} />
            <Route path='*' element={<NotFoundPage />} />
          </Route>
        </Routes>
      </ErrorBoundary>
    </BrowserRouter>
  );
};

export default App;
