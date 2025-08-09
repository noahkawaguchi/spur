import SinglePostDisplay from '@/components/PostsDisplay/SinglePostDisplay';
import useRequest from '@/hooks/useRequest';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import { PostArraySchema, type Post } from '@/types';
import { useEffect, useState } from 'react';
import styles from '@/styles/shared.module.css';
import { first100chars, howLongAgo } from '@/utils/fmt';

const PostsDisplay = ({
  header,
  endpoint,
  displayUsernames,
}: {
  header: React.ReactElement;
  endpoint: string;
  displayUsernames: boolean;
}) => {
  const token = useTokenOrRedirect();
  const [readingPost, setReadingPost] = useState<Post | null>(null);
  const {
    data: posts,
    error,
    loading,
    sendRequest,
  } = useRequest<null, Post[]>({ method: 'GET', endpoint, respSchema: PostArraySchema });

  useEffect(() => {
    if (token) void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      {!readingPost && header}
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {posts &&
        (readingPost ? (
          <SinglePostDisplay readingPost={readingPost} setReadingPost={setReadingPost} />
        ) : (
          <>
            {posts.length ? (
              <table>
                <tbody>
                  {posts.map(post => (
                    <tr key={post.id}>
                      {displayUsernames && <th>by {post.authorUsername}</th>}
                      <td>created {howLongAgo(post.createdAtMs)} ago</td>
                      <td>{first100chars(post.body)}</td>
                      <td className={styles.buttonCell}>
                        <button
                          type='button'
                          onClick={() => {
                            setReadingPost(post);
                          }}
                        >
                          Read
                        </button>
                      </td>
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

export default PostsDisplay;
