import styles from './Footer.module.css';

const Footer = () => {
  return (
    <footer className={styles.universalFooter}>
      <p>
        See this project's repository on
        <a href='https://github.com/noahkawaguchi/spur' target='_blank' rel='noopener noreferrer'>
          GitHub
        </a>
      </p>
    </footer>
  );
};

export default Footer;
