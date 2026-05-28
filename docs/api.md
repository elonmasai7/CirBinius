# API Reference

Planned endpoints:

- `POST /api/projects`
- `POST /api/projects/:id/upload`
- `POST /api/compile`
- `GET /api/jobs/:id`
- `GET /api/jobs/:id/logs`
- `GET /api/jobs/:id/artifacts`
- `POST /api/prove`
- `POST /api/verify`
- `POST /api/analyze`
- `GET /api/health`
- `GET /api/metrics`

API jobs are asynchronous and execute in sandboxed workers.
