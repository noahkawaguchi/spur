import useRequest from '../../hooks/useRequest';
import { type SuccessResponse, SuccessResponseSchema, type AddFriendRequest } from '../../types';
import { useTokenOrRedirect } from '../../utils/jwt';

const FriendRequest = ({ username }: { username: string }) => {
  const token = useTokenOrRedirect();

  const { data, error, loading, sendRequest } = useRequest<SuccessResponse, AddFriendRequest>(
    'POST',
    'friends',
    SuccessResponseSchema,
  );

  const handleClick = (e: React.MouseEvent) => {
    e.preventDefault();
    void sendRequest({ token, body: { recipientUsername: username } });
  };

  return (
    <>
      <tr>
        <td>{username}</td>
        <button type='button' onClick={handleClick}>
          Accept
        </button>
      </tr>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && <p>{data.message}</p>}
    </>
  );
};

export default FriendRequest;
