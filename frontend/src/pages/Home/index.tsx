import { Link } from 'react-router-dom';
import styles from './HomePage.module.css';

const HomePage = () => {
  return (
    <>
      <h1>Welcome to Spur!</h1>
      <div className={styles.jockeyBackground}>
        <div className={styles.jockeyFlipper}>
          <h2 className={styles.jockey}>🏇🏻</h2>
        </div>
      </div>
      <hr />
      <p>Spur is a reply-based social platform.</p>
      <p>
        Check out the code, documentation, and feature demonstrations on
        <a href='https://github.com/noahkawaguchi/spur' target='_blank' rel='noopener noreferrer'>
          <button type='button'>GitHub</button>
        </a>
        or
        <Link to='/feed'>
          <button type='button'>get started</button>
        </Link>
        with your own account.
      </p>
      <hr />
    </>
  );
};

export default HomePage;
