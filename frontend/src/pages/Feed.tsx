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
      endpoint='posts/friends'
      displayUsernames={true}
    />
  );
};

export default FeedPage;
