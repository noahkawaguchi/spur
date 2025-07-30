import LoginPage from '@/pages/Login';
import { inMemRouter } from '@/test-utils/router';
import { render, screen } from '@testing-library/react';

describe('LoginPage', () => {
  it('should render', () => {
    render(inMemRouter({ children: <LoginPage /> }));
    expect(screen.getByText('Login')).toBeInTheDocument();
  });
});
