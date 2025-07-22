import { useEffect } from 'react';
import useRequest from '../hooks/useRequest';
import { ContentSchema, type Content } from '../types';
import { useTokenOrRedirect } from '../utils/jwt';

const ContentDisplay = ({ endpoint }: { endpoint: string }) => {
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
          <h3>
            <i>Prompts</i>
          </h3>
          {data.prompts.length ? (
            data.prompts.map(prompt => (
              <div key={prompt.id}>
                <hr />
                <h4>
                  Prompt {prompt.id} (by {prompt.authorUsername})
                </h4>
                <p>{prompt.body}</p>
              </div>
            ))
          ) : (
            <p>(No prompts)</p>
          )}
          <hr />
          <h3>
            <i>Posts</i>
          </h3>
          {data.posts.length ? (
            data.posts.map(post => (
              <div key={post.id}>
                <hr />
                <h4>
                  Post {post.id} (by {post.authorUsername})
                </h4>
                <p>
                  <i>
                    In response to prompt {post.prompt.id} (by {post.prompt.authorUsername}): "
                    {post.prompt.body}"
                  </i>
                </p>
                <p>{post.body}</p>
              </div>
            ))
          ) : (
            <p>(No posts)</p>
          )}
        </>
      )}
    </>
  );
};

export default ContentDisplay;
