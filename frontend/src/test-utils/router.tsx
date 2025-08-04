import { MemoryRouter } from 'react-router-dom';

export const inMemRouter = ({ children }: { children: React.ReactNode }) => (
  <MemoryRouter>{children}</MemoryRouter>
);
