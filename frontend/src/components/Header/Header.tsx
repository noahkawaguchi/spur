import { Link } from 'react-router-dom';
import styles from './Header.module.css';

const Header = () => {
  return (
    <header>
      <h1 className={styles.titleHeading}>
        <Link to='/' className={styles.titleLink}>
          Spur
        </Link>
      </h1>
      <nav>
        <Link to='/feed'>
          <button type='button'>Feed</button>
        </Link>
        <Link to='/create'>
          <button type='button'>Create</button>
        </Link>
        <Link to='/friends'>
          <button type='button'>Friends</button>
        </Link>
        <Link to='/profile'>
          <button type='button'>Profile</button>
        </Link>
      </nav>
    </header>
  );
};

export default Header;
