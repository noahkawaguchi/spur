import { Link } from 'react-router-dom';

const NotFoundPage = () => {
  return (
    <>
      <h2>404 - Page Not Found</h2>
      <Link to='/'>
        <button type='button'>Back home</button>
      </Link>
    </>
  );
};

export default NotFoundPage;
