import { Link } from 'react-router-dom';

const NotFoundPage = () => {
  return (
    <>
      <h3>404 - Page Not Found</h3>
      <Link to='/'>Back to home</Link>
    </>
  );
};

export default NotFoundPage;
