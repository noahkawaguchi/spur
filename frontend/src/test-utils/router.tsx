import { render } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';

export const renderInMemRouter = (el: React.ReactElement) => {
  render(<MemoryRouter>{el}</MemoryRouter>);
};
