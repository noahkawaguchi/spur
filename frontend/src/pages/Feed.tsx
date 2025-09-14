import PostsDisplay from '@/components/PostsDisplay';

const FeedPage = () => {
  return (
    <PostsDisplay
      header={
        <>
          <h2>Feed</h2>
          <hr />
        </>
      }
      endpoint='friends/posts'
      displayUsernames={true}
    />
  );
};

export default FeedPage;
