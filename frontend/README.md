# EventDesign Frontend

React + TypeScript + Vite client for EventDesign.

## Commands

```bash
npm install
npm run dev
npm run lint
npm run build
```

## Environment

Copy `.env.example` to `.env`.

Default local values:

```bash
VITE_API_BASE_URL=/api
VITE_EDGE_API_ORIGIN=http://localhost:8080
```

The development server proxies `/api` to `VITE_EDGE_API_ORIGIN`, so browser auth can stay on the cookie-based flow without storing tokens in the client.
