import { Outlet } from 'react-router-dom';
import Footer from '@/components/Footer/Footer';

const UniversalLayout = () => {
  return (
    <>
      <Outlet />
      <Footer />
    </>
  );
};

export default UniversalLayout;
