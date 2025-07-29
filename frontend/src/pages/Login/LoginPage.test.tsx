import LoginPage from '@/pages/Login';
import { renderInMemRouter } from '@/test-utils/router';
import { screen } from '@testing-library/react';

describe('LoginPage', () => {
  it('should render', () => {
    renderInMemRouter(<LoginPage />);
    expect(screen.getByText('Login')).toBeInTheDocument();
  });
});
