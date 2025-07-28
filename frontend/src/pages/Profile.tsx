import ContentDisplay from '@/components/ContentDisplay';
import { removeToken } from '@/utils/jwt';
import { useNavigate } from 'react-router-dom';

const ProfilePage = () => {
  const navigate = useNavigate();

  const handleLogout = () => {
    removeToken();
    void navigate('/');
  };

  return (
    <ContentDisplay
      header={
        <>
          <h2>Your profile</h2>
          <button type='button' onClick={handleLogout}>
            Log out
          </button>
          <hr />
        </>
      }
      endpoint='content/me'
      displayUsername={false}
    />
  );
};

export default ProfilePage;
