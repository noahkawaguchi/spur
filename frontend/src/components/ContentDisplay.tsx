import { useEffect, useState } from 'react';
import useRequest from '../hooks/useRequest';
import { ContentSchema, type Content, type Post, type Prompt } from '../types';
import { useTokenOrRedirect } from '../utils/jwt';
import PostReader from './PostReaderWriter/PostReader';
import PostWriter from './PostReaderWriter/PostWriter';

const ContentDisplay = ({
  header,
  endpoint,
  displayUsername,
}: {
  header: React.ReactElement;
  endpoint: string;
  displayUsername: boolean;
}) => {
  const token = useTokenOrRedirect();
  const [readingPost, setReadingPost] = useState<Post | null>(null);
  const [respondingToPrompt, setRespondingToPrompt] = useState<Prompt | null>(null);
  const { data, error, loading, sendRequest } = useRequest<null, Content>({
    method: 'GET',
    endpoint,
    respSchema: ContentSchema,
  });

  useEffect(() => {
    void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      {!readingPost && !respondingToPrompt && header}
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data &&
        (readingPost ? (
          <PostReader post={readingPost} setReadingPost={setReadingPost} />
        ) : respondingToPrompt ? (
          <PostWriter prompt={respondingToPrompt} setRespondingToPrompt={setRespondingToPrompt} />
        ) : (
          <>
            <h3>Prompts</h3>
            {data.prompts.length ? (
              <table>
                <tbody>
                  {data.prompts.map(prompt => (
                    <tr key={prompt.id}>
                      {displayUsername && <th>by {prompt.authorUsername}</th>}
                      <td>{prompt.body}</td>
                      <td className='button-cell'>
                        <button
                          type='button'
                          onClick={() => {
                            setRespondingToPrompt(prompt);
                          }}
                        >
                          Write post
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <p>(No prompts)</p>
            )}
            <h3>Posts</h3>
            {data.posts.length ? (
              <table>
                <tbody>
                  {data.posts.map(post => (
                    <tr key={post.id}>
                      {displayUsername && <th>by {post.authorUsername}</th>}
                      <td>
                        in response to {post.prompt.authorUsername}: "{post.prompt.body}"
                      </td>
                      <td className='button-cell'>
                        {' '}
                        <button
                          type='button'
                          onClick={() => {
                            setReadingPost(post);
                          }}
                        >
                          Read
                        </button>
                      </td>{' '}
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <p>(No posts)</p>
            )}
          </>
        ))}
    </>
  );
};

export default ContentDisplay;
