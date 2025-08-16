import PostsDisplay from '@/components/PostsDisplay';
import { Link, useParams } from 'react-router-dom';

const FriendContentPage = () => {
  const { username } = useParams<{ username: string }>();
  if (!username) throw new Error('unexpected undefined username in FriendsContentPage');

  return (
    <PostsDisplay
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
