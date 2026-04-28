# gpt-copy-v6

Generated CodeDungeon example repository for a ChatGPT-style application.

## Local Environment

Create a local .env file from .env.example and set:

`dotenv
OPENROUTER_API_KEY=<local secret>
OPENROUTER_MODEL=nvidia/nemotron-3-super-120b-a12b:free
DATABASE_URL=sqlite://gpt-copy.db
BACKEND_PORT=8080
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080
`

Do not commit real provider keys.

## CodeDungeon

Project Rules must be approved before the first real run:

`powershell
.\.codex\bin\codedungeon.exe rules status --human
`

The initial full-run prompt lives in prompts/full-v6.txt.