import type { Post } from '../types';

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
        <h2>Post {post.id}</h2>
        <h3>by {post.authorUsername}</h3>
        <p>
          <i>
            in response to {post.prompt.authorUsername}: "{post.prompt.body}"
          </i>
        </p>
      </div>
      <hr />
      <p>{post.body}</p>
    </>
  );
};

export default PostReader;
