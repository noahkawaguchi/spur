import { useEffect } from 'react';
import useRequest from '../hooks/useRequest';
import { ContentSchema, type Content } from '../types';
import { useTokenOrRedirect } from '../utils/jwt';

const ContentDisplay = ({
  endpoint,
  displayUsername,
}: {
  endpoint: string;
  displayUsername: boolean;
}) => {
  const token = useTokenOrRedirect();
  const { data, error, loading, sendRequest } = useRequest<Content>('GET', endpoint, ContentSchema);

  useEffect(() => {
    void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && (
        <>
          <h3>Prompts</h3>
          {data.prompts.length ? (
            <table>
              {data.prompts.map(prompt => (
                <tr key={prompt.id}>
                  {displayUsername && <th>by {prompt.authorUsername}</th>}
                  <td>{prompt.body}</td>
                </tr>
              ))}
            </table>
          ) : (
            <p>(No prompts)</p>
          )}
          <h3>Posts</h3>
          {data.posts.length ? (
            <table>
              {data.posts.map(post => (
                <tr key={post.id}>
                  {displayUsername && <th>by {post.authorUsername}</th>}
                  <td>
                    in response to {post.prompt.authorUsername}: "{post.prompt.body}"
                  </td>
                  <td>{post.body}</td>
                </tr>
              ))}
            </table>
          ) : (
            <p>(No posts)</p>
          )}
        </>
      )}
    </>
  );
};

export default ContentDisplay;
