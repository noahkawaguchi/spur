import useRequest from '@/hooks/useRequest';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import { SuccessResponseSchema, type AddFriendRequest, type SuccessResponse } from '@/types';
import styles from '@/styles/shared.module.css';

const FriendRequest = ({ username }: { username: string }) => {
  const token = useTokenOrRedirect();

  const { data, error, loading, sendRequest } = useRequest<AddFriendRequest, SuccessResponse>({
    method: 'POST',
    endpoint: 'friends',
    respSchema: SuccessResponseSchema,
  });

  const handleClick = () => {
    if (token) void sendRequest({ token, body: { recipientUsername: username } });
  };

  return (
    <>
      <tr>
        <td>{username}</td>
        <td className={styles.buttonCell}>
          <button type='button' onClick={handleClick}>
            Accept
          </button>
        </td>
      </tr>
      <tr>
        <td style={{ border: 'none', background: 'none' }}>
          {loading && <p>Loading...</p>}
          {error && <p>Error: {error}</p>}
          {data && <p>{data.message}</p>}
        </td>
      </tr>
    </>
  );
};

export default FriendRequest;
