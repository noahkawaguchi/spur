import { Outlet, useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { getToken } from '@/utils/jwt';
import Header from '@/components/Header/Header';

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
