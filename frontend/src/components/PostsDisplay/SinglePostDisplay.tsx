import type { Post } from '@/types';
import styles from './PostsDisplay.module.css';
import { howLongAgo } from '@/utils/fmt';
import { useState } from 'react';
import ReplyWriter from './ReplyWriter';

// TODO: add buttons to get parent post and children posts

const SinglePostDisplay = ({ readingPost, backFn }: { readingPost: Post; backFn: () => void }) => {
  const [replying, setReplying] = useState(false);

  return (
    <>
      <button type='button' onClick={backFn}>
        Back
      </button>
      <div style={{ textAlign: 'center' }}>
        <h2>Post by {readingPost.authorUsername}</h2>
        <p>created {howLongAgo(readingPost.createdAtMs)} ago</p>
        <hr />
        <p className={styles.postBodyArea} style={{ whiteSpace: 'pre-line', textAlign: 'justify' }}>
          {readingPost.body}
        </p>
        {replying ? (
          <ReplyWriter
            parentId={readingPost.id}
            cancelFn={() => {
              setReplying(false);
            }}
          />
        ) : (
          <button
            type='button'
            onClick={() => {
              setReplying(true);
            }}
          >
            Reply
          </button>
        )}
      </div>
    </>
  );
};

export default SinglePostDisplay;
