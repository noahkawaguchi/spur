import type { Post } from '@/types';
import styles from './PostReaderWriter.module.css';

const PostReader = ({
  post,
  setReadingPost,
}: {
  post: Post;
  setReadingPost: (post: Post | null) => void;
}) => {
  return (
    <>
      <button
        type='button'
        onClick={() => {
          setReadingPost(null);
        }}
      >
        Back
      </button>
      <div style={{ textAlign: 'center' }}>
        <h2>Post by {post.authorUsername}</h2>
        <p>
          <i>
            in response to {post.prompt.authorUsername}: "{post.prompt.body}"
          </i>
        </p>
      </div>
      <hr />
      <p className={styles.postReadingArea}>{post.body}</p>
    </>
  );
};

export default PostReader;
