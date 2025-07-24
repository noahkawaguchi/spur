import { useParams } from 'react-router-dom';
import ContentDisplay from '../../components/ContentDisplay';

const FriendContentPage = () => {
  const { username } = useParams<{ username: string }>();
  if (!username) throw new Error('unexpected undefined username in FriendsContentPage');

  return (
    <ContentDisplay
      header={
        <>
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
