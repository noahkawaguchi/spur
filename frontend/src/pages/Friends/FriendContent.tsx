import ContentDisplay from '@/components/ContentDisplay';
import { Link, useParams } from 'react-router-dom';

const FriendContentPage = () => {
  const { username } = useParams<{ username: string }>();
  if (!username) throw new Error('unexpected undefined username in FriendsContentPage');

  return (
    <ContentDisplay
      header={
        <>
          <Link to='/friends'>
            <button type='button'>Back</button>
          </Link>
          <h2>{username}</h2>
          <hr />
        </>
      }
      endpoint={`content/friend/${username}`}
      displayUsername={false}
    />
  );
};

export default FriendContentPage;
