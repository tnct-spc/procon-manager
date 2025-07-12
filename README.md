# Procon-Manager

A modern, full-stack item management system built with Rust and Vue.js, designed for tracking and managing various types of items with user authentication and checkout functionality.

## Features

- **Multi-type Item Management**: Support for books (with ISBN), laptops (with MAC address), and general items
- **User Authentication**: JWT-based authentication with role-based access control (Admin/User)
- **Checkout System**: Track item borrowing and returns with timestamps
- **RESTful API**: OpenAPI 3.0 documented API with Swagger UI
- **Modern Frontend**: Responsive Vue.js interface with TypeScript
- **Clean Architecture**: Layered architecture with clear separation of concerns

## Tech Stack

### Backend

- **Rust** with Axum web framework
- **PostgreSQL** with SQLX for type-safe database queries
- **JWT** authentication with bcrypt password hashing
- **OpenAPI 3.0** documentation with Swagger UI

### Frontend

- **Vue.js 3** with Composition API
- **TypeScript** for type safety
- **Vite** for fast development and building
- **Pinia** for state management

### Infrastructure

- **Nix Flakes** for reproducible development environment
- **Process Compose** for service orchestration
- **GitHub Actions** for CI/CD

## Quick Start

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled

### Development Setup

1. **Enter development environment**

   ```bash
   nix develop
   ```

2. **Start PostgreSQL**

   ```bash
   nix run .#dev -- -D
   ```

3. **Access the application**
   - Frontend: <http://localhost:5173>
   - Backend API: <http://localhost:8081>
   - Swagger UI: <http://localhost:8081/swagger-ui/>

### Manual Development

If you prefer to run services separately:

1. **Start database and backend**

   ```bash
   cargo make compose-up
   cd item-manager
   ```

2. **Start frontend** (in another terminal)

   ```bash
   cd frontend
   pnpm install
   pnpm dev
   ```

## API Usage

The REST API is documented with OpenAPI 3.0. Access the interactive documentation at `/swagger-ui/` when the backend is running.

### Authentication

```bash
# Login
curl http://localhost:8081/auth/login --json '{"email": "admin@example.com", "password": "password"}'

# Use the returned JWT token in subsequent requests
curl http://localhost:8081/api/v1/items -H "Authorization: Bearer <your-jwt-token>"
```

## Development Commands

### Backend (item-manager/)

```bash
cargo make watch      # Development with auto-reload
cargo make test-ci    # Run tests with database setup
cargo make migrate    # Run database migrations
cargo make psql       # Connect to PostgreSQL
```

### Frontend (frontend/)

```bash
pnpm dev         # Development server
pnpm build       # Build for production
pnpm lint        # ESLint with auto-fix
pnpm format      # Prettier formatting
```

### Quality Assurance

```bash
nix flake check -L              # Full CI checks
```

## Project Structure

```
procon-manager/
├── item-manager/           # Rust backend (Clean Architecture)
│   ├── kernel/            # Domain layer (business logic)
│   ├── adapter/           # Infrastructure layer (database)
│   ├── api/               # Presentation layer (HTTP)
│   ├── shared/            # Cross-cutting concerns
│   └── registry/          # Dependency injection
├── frontend/              # Vue.js frontend
│   ├── src/
│   │   ├── components/    # Vue components
│   │   ├── views/         # Route components
│   │   ├── services/      # API integration
│   │   └── stores/        # Pinia state management
├── flake.nix             # Nix development environment
└── README.md
```
