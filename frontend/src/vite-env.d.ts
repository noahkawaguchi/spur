/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BACKEND_URL: string;
  // Add other env vars here as needed
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
