# gpt-copy-v6

ChatGPT-style local application with a Rust 2024 Axum backend and a Next.js App Router frontend.

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: be792d27d9969ae5ed95a5caa5de79b0b97d20bbd97ec73bf92cd031941e9d31
PROJECT_RULES_READ: yes

## Repository Layout

- `backend/`: Axum API, SQLite persistence through sqlx, OpenRouter server-side provider, backend tests.
- `frontend/`: Next.js App Router, TypeScript, Tailwind, chat UI, component tests, Playwright smoke test.
- `.env.example`: placeholder-only local environment template.

## Setup

Run setup from the repository root. Install a standard Rust toolchain and Node.js/npm first, then install the locked frontend dependencies and create a local environment file:

```powershell
rustc --version
cargo --version
node --version
npm --version
npm --prefix frontend ci
Copy-Item .env.example .env
```

The backend uses Cargo directly from `backend/`; no generated build output or local database should be committed.

## Environment

Keep real secrets only in ignored local `.env` files. OpenRouter calls must stay server-side in the backend.

```dotenv
OPENROUTER_API_KEY=
OPENROUTER_MODEL=
OPENROUTER_HTTP_REFERER=http://localhost:3000
OPENROUTER_TITLE=gpt-copy-v6
DATABASE_URL=sqlite:gpt-copy-v6.db
BACKEND_BIND_ADDR=127.0.0.1:8080
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080
```

`OPENROUTER_API_KEY` is required for real provider calls. Leave `OPENROUTER_MODEL` empty or unset to use the backend default: `nvidia/nemotron-3-super-120b-a12b:free`.

## Backend Run

```powershell
cargo run --manifest-path backend/Cargo.toml
```

Health check:

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
```

## Frontend Run

```powershell
npm --prefix frontend run dev
```

Open `http://localhost:3000`. The frontend defaults to `http://localhost:8080` when `NEXT_PUBLIC_API_BASE_URL` is empty or unset.

## Tests

Backend verification from the repository root:

```powershell
cargo fmt --manifest-path backend/Cargo.toml --all -- --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml
cargo build --manifest-path backend/Cargo.toml
```

Frontend verification from the repository root:

```powershell
npm --prefix frontend run lint
npm --prefix frontend run test
npm --prefix frontend run build
npm --prefix frontend run test:e2e
```

Automated tests should use mocks and isolated SQLite databases instead of real OpenRouter quota.

## API

- `GET /health`
- `GET /conversations`
- `POST /conversations`
- `GET /conversations/:id/messages`
- `POST /conversations/:id/messages`
- `POST /conversations/:id/messages/stream`

The streaming endpoint returns `text/event-stream` events: `message_start`, `delta`, `message_complete`, and `error`.

## Troubleshooting

- Confirm the backend is reachable:

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
```

- Check whether the backend or frontend ports are already in use:

```powershell
Get-NetTCPConnection -LocalPort 8080,3000 -ErrorAction SilentlyContinue | Select-Object LocalAddress,LocalPort,OwningProcess
```

- Reinstall locked frontend dependencies:

```powershell
npm --prefix frontend ci
```

- Install the Playwright Chromium browser:

```powershell
npm --prefix frontend exec playwright install chromium
```

- Re-run the smoke test after the backend and frontend are available:

```powershell
npm --prefix frontend run test:e2e
```

`OpenRouter API key is missing` means `OPENROUTER_API_KEY` is not set in local `.env`. `database is locked` usually means another backend process is holding the SQLite file; stop the duplicate process and retry.

## Secret Handling

Tracked files contain placeholders only. Keep real OpenRouter keys in local `.env` files.
