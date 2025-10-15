import { PostSchema, type Post } from '@/types';
import styles from './PostsDisplay.module.css';
import { firstChars, howLongAgo } from '@/utils/fmt';
import { useState } from 'react';
import ReplyWriter from './ReplyWriter';
import useRequest from '@/hooks/useRequest';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import PostsDisplay from '.';

const SinglePostDisplay = ({ readingPost, backFn }: { readingPost: Post; backFn: () => void }) => {
  const token = useTokenOrRedirect();
  const [replying, setReplying] = useState(false);
  const [nestedParent, setNestedParent] = useState(false);
  const [nestedChildren, setNestedChildren] = useState(false);

  const {
    data: parentPost,
    loading,
    error,
    sendRequest,
  } = useRequest<null, Post>({ method: 'GET', endpoint: 'posts', respSchema: PostSchema });

  const handleParentClick = () => {
    if (token && readingPost.parentId) {
      void sendRequest({ token, pathParameter: readingPost.parentId.toString() });
      setNestedParent(true);
    }
  };

  return nestedParent ? (
    <>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {parentPost && (
        <SinglePostDisplay readingPost={parentPost} backFn={() => setNestedParent(false)} />
      )}
    </>
  ) : nestedChildren ? (
    <PostsDisplay
      header={
        <>
          <button type='button' onClick={() => setNestedChildren(false)}>
            Back
          </button>
          <h2>Children of {readingPost.authorUsername}'s Post</h2>
          <p>
            <i>"{firstChars(readingPost.body, 100)}"</i>
          </p>
          <hr />
        </>
      }
      endpoint={`posts/${readingPost.id.toString()}/children`}
      displayUsernames={true}
    />
  ) : (
    <>
      <button type='button' onClick={backFn}>
        Back
      </button>
      <div style={{ textAlign: 'center' }}>
        <h2>Post by {readingPost.authorUsername}</h2>
        <p>created {howLongAgo(readingPost.createdAtMs)} ago</p>
        {readingPost.editedAtMs && <p>edited {howLongAgo(readingPost.editedAtMs)} ago</p>}
        <hr />
        <p className={styles.postBodyArea} style={{ whiteSpace: 'pre-line', textAlign: 'justify' }}>
          {readingPost.body}
        </p>
        {replying ? (
          <ReplyWriter parentId={readingPost.id} cancelFn={() => setReplying(false)} />
        ) : (
          <>
            <button type='button' onClick={() => setReplying(true)}>
              Reply
            </button>
            {readingPost.parentId && (
              <button type='button' onClick={handleParentClick}>
                Parent
              </button>
            )}
            <button type='button' onClick={() => setNestedChildren(true)}>
              Children
            </button>
          </>
        )}
      </div>
    </>
  );
};

export default SinglePostDisplay;
