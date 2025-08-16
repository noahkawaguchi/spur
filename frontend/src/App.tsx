import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';
import ErrorFallbackPage from '@/pages/ErrorFallback';
import LoginPage from '@/pages/Login';
import SignupPage from '@/pages/Signup';
import MainLayout from '@/Layouts/MainLayout';
import FeedPage from '@/pages/Feed';
import NotFoundPage from '@/pages/NotFound';
import FriendsPage from '@/pages/Friends';
import FriendRequestsPage from '@/pages/Friends/Requests';
import AddFriendPage from '@/pages/Friends/Add';
import FriendContentPage from '@/pages/Friends/FriendContent';
import ProfilePage from '@/pages/Profile';
import './App.css';
import UniversalLayout from '@/Layouts/UniversalLayout';

const App = () => {
  return (
    <BrowserRouter>
      <ErrorBoundary FallbackComponent={ErrorFallbackPage}>
        <Routes>
          <Route element={<UniversalLayout />}>
            <Route path='/login' element={<LoginPage />} />
            <Route path='/signup' element={<SignupPage />} />
            <Route element={<MainLayout />}>
              <Route path='/' element={<FeedPage />} />
              <Route path='/friends' element={<FriendsPage />} />
              <Route path='/friends/requests' element={<FriendRequestsPage />} />
              <Route path='/friends/add' element={<AddFriendPage />} />
              <Route path='/friends/:username' element={<FriendContentPage />} />
              <Route path='/profile' element={<ProfilePage />} />
              <Route path='*' element={<NotFoundPage />} />
            </Route>
          </Route>
        </Routes>
      </ErrorBoundary>
    </BrowserRouter>
  );
};

export default App;
