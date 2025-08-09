import ManyPostsDisplay from '@/components/ManyPostsDisplay';
import { Link, useParams } from 'react-router-dom';

const FriendContentPage = () => {
  const { username } = useParams<{ username: string }>();
  if (!username) throw new Error('unexpected undefined username in FriendsContentPage');

  return (
    <ManyPostsDisplay
      header={
        <>
          <Link to='/friends'>
            <button type='button'>Back</button>
          </Link>
          <h2>{username}</h2>
          <hr />
        </>
      }
      endpoint={`posts/user/${username}`}
      displayUsernames={false}
    />
  );
};

export default FriendContentPage;
