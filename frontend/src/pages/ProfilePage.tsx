import { useNavigate } from 'react-router-dom';
import { removeToken } from '../utils/jwt';
import ContentDisplay from '../components/ContentDisplay';

const ProfilePage = () => {
  const navigate = useNavigate();

  const handleLogout = () => {
    removeToken();
    void navigate('/');
  };

  return (
    <>
      <h2>Your profile</h2>
      <button type='button' onClick={handleLogout}>
        Log out
      </button>
      <hr />
      <ContentDisplay endpoint='content/me' displayUsername={false} />
    </>
  );
};

export default ProfilePage;
