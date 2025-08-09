import ManyPostsDisplay from '@/components/ManyPostsDisplay';
import { removeToken } from '@/utils/jwt';
import { useNavigate } from 'react-router-dom';

const ProfilePage = () => {
  const navigate = useNavigate();

  const handleLogout = () => {
    removeToken();
    void navigate('/');
  };

  return (
    <ManyPostsDisplay
      header={
        <>
          <h2>Your profile</h2>
          <button type='button' onClick={handleLogout}>
            Log out
          </button>
          <hr />
        </>
      }
      endpoint='posts/me'
      displayUsernames={false}
    />
  );
};

export default ProfilePage;
