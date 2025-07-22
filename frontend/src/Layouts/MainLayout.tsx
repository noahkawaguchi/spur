import { Outlet, useNavigate } from 'react-router-dom';
import Header from '../components/Header/Header';
import { getToken } from '../utils/jwt';
import { useEffect } from 'react';

const MainLayout = () => {
  const navigate = useNavigate();

  useEffect(() => {
    if (!getToken()) void navigate('/login');
  }, [navigate]);

  return (
    <>
      <Header />
      <hr />
      <main>
        <Outlet />
      </main>
      <hr />
    </>
  );
};

export default MainLayout;
