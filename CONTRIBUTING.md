# Contributing to Tycoon Monorepo

Thanks for contributing! This guide covers local setup and the workflow we use for pull requests, with a focus on the `frontend/` app.

## Repository layout

- `frontend/` — Next.js app (React 19, TypeScript, Vitest, Storybook)
- `backend/` — NestJS API
- `contract/` — Soroban smart contracts
- `shop-api/` — shop service

## Frontend setup

Requirements: **Node 20** (matches the version pinned in `.github/workflows/frontend-ci.yml`).

```bash
cd frontend
npm ci --legacy-peer-deps
```

`--legacy-peer-deps` is required — the frontend's dependency tree has peer dependency ranges that `npm`'s default resolver rejects.

Common commands, run from `frontend/`:

```bash
npm run dev             # start the dev server
npm run build            # production build (also type-checks via `next build`)
npm run typecheck        # tsc --noEmit
npm run lint              # eslint
npm run lint:ci           # non-mutating lint for files changed in the commit
npm test -- --run         # run the Vitest suite once (CI mode)
npm run test:coverage     # Vitest with coverage
npm run storybook         # Storybook dev server
npm run build-storybook   # static Storybook build
```

Before opening a PR that touches `frontend/`, make sure `npm test -- --run`, `npm run typecheck`, and `npm run build` all pass locally — these are the checks enforced by [Frontend CI](.github/workflows/frontend-ci.yml).

## Workflow

1. Create a branch off `main`: `feature/<issue-number>-short-description` or `fix/<issue-number>-short-description`.
2. Implement the change, adding or updating tests alongside it.
3. Run the relevant checks for the part of the repo you touched (see above for frontend; `backend/` and `contract/` have their own `npm`/`make` scripts).
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/) (`feat(...)`, `fix(...)`, `docs(...)`, etc.).
5. Open a PR against `main` using the PR template, referencing the issue with `closes #<issue-number>`.

## Picking up your first issue

New to the codebase? Start with issues labeled [`good first issue`](https://github.com/SaboStudios/Tycoon-Monorepo/labels/good%20first%20issue) — these are scoped to a single file or small area. Once you're comfortable with the codebase conventions, move on to [`help wanted`](https://github.com/SaboStudios/Tycoon-Monorepo/labels/help%20wanted) issues, which are larger or touch more of the system. Issues are also labeled by area (`frontend`, `backend`, `contract`) to help you find ones matching your experience.
