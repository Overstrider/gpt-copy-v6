# gpt-copy-v6

ChatGPT-style local application with a Rust 2024 Axum backend and a Next.js App Router frontend.

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

## Repository Layout

- `backend/`: Axum API, SQLite persistence through sqlx, OpenRouter server-side provider, backend tests.
- `frontend/`: Next.js App Router, TypeScript, Tailwind, chat UI, component tests, Playwright smoke test.
- `.env.example`: placeholder-only local environment template.

## Environment

Copy `.env.example` to `.env` for local development and set a real OpenRouter key only in `.env`.

```dotenv
OPENROUTER_API_KEY=
OPENROUTER_MODEL=nvidia/nemotron-3-super-120b-a12b:free
OPENROUTER_HTTP_REFERER=http://localhost:3000
OPENROUTER_TITLE=gpt-copy-v6
DATABASE_URL=sqlite:gpt-copy-v6.db
BACKEND_BIND_ADDR=127.0.0.1:8080
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080
```

Never commit a real provider key. OpenRouter requests are made by the backend only.

## Backend

```powershell
Push-Location backend
cargo run
Pop-Location
```

Health check:

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
```

Backend verification:

```powershell
Push-Location backend
cargo fmt -- --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
Pop-Location
```

## Frontend

```powershell
Push-Location frontend
npm install
npm run dev
Pop-Location
```

Open `http://localhost:3000`.

Frontend verification:

```powershell
Push-Location frontend
npm run lint
npm run test
npm run build
npx playwright test
Pop-Location
```

## Local Development

Use two terminals:

```powershell
Push-Location backend
cargo run
```

```powershell
Push-Location frontend
npm run dev
```

The frontend defaults to `http://localhost:8080` when `NEXT_PUBLIC_API_BASE_URL` is not set. Set it in `frontend/.env.local` only when the backend uses a different origin. The backend reads `OPENROUTER_API_KEY`, `OPENROUTER_MODEL`, `DATABASE_URL`, and `BACKEND_BIND_ADDR`.

## API

- `GET /health`
- `GET /conversations`
- `POST /conversations`
- `GET /conversations/:id/messages`
- `POST /conversations/:id/messages`
- `POST /conversations/:id/messages/stream`

The streaming endpoint returns `text/event-stream` events: `message_start`, `delta`, `message_complete`, and `error`.

## Troubleshooting

- `OpenRouter API key is missing`: set `OPENROUTER_API_KEY` in local `.env`.
- `database is locked`: stop duplicate backend processes and retry; tests use isolated SQLite databases.
- Frontend API errors: confirm the backend is running on `BACKEND_BIND_ADDR` and `NEXT_PUBLIC_API_BASE_URL` points to it.
- Playwright browser missing: run `npx playwright install chromium` from `frontend/`.

## Secret Handling

Tracked files contain placeholders only. Keep real OpenRouter keys in local `.env` files.
