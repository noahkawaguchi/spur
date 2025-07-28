import ContentDisplay from '@/components/ContentDisplay';

const FeedPage = () => {
  return (
    <ContentDisplay
      header={
        <>
          <h2>Feed</h2>
          <hr />
        </>
      }
      endpoint='content'
      displayUsername={true}
    />
  );
};

export default FeedPage;
