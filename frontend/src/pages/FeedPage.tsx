import ContentDisplay from '../components/ContentDisplay';

const FeedPage = () => {
  return (
    <>
      <>
        <h2>Feed</h2>
        <hr />
        <ContentDisplay endpoint='content' displayUsername={true} />
      </>
    </>
  );
};

export default FeedPage;
