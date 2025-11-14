import { Link } from 'react-router-dom';
import styles from './Header.module.css';

const Header = () => {
  return (
    <header>
      <Link to='/' style={{ textDecoration: 'none', cursor: 'default' }}>
        <h1 className={styles.headerTitle}>Spur</h1>
      </Link>
      <nav>
        <Link to='/feed'>
          <button type='button'>Feed</button>
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
