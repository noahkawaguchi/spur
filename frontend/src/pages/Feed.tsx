import ManyPostsDisplay from '@/components/ManyPostsDisplay';

const FeedPage = () => {
  return (
    <ManyPostsDisplay
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
