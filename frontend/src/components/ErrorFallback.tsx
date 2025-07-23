import type { FallbackProps } from 'react-error-boundary';
import { useNavigate } from 'react-router-dom';

const ErrorFallback = ({ error, resetErrorBoundary }: FallbackProps) => {
  const navigate = useNavigate();

  console.log(error);

  const handleClick = () => {
    resetErrorBoundary();
    void navigate('/');
  };

  return (
    <>
      <h2>Unexpected error! (⸝⸝⸝&gt;﹏&lt;⸝⸝⸝)</h2>
      <button type='button' onClick={resetErrorBoundary}>
        Retry
      </button>
      <button type='button' onClick={handleClick}>
        Back home
      </button>
    </>
  );
};

export default ErrorFallback;
