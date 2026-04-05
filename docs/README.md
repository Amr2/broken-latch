# broken-latch Platform

Game overlay platform built with Tauri 2.0 + React + TypeScript.

## Project Structure

```
broken-latch/
├── src/                      # React frontend
│   ├── components/           # UI components
│   ├── App.tsx              # Main app component
│   └── main.tsx             # Entry point
├── src-tauri/               # Rust backend
│   ├── src/                 # Rust source code
│   │   ├── main.rs         # Tauri entry point
│   │   ├── config.rs       # Configuration management
│   │   ├── db.rs           # Database operations
│   │   ├── overlay/        # Overlay window system
│   │   ├── game/           # Game lifecycle detection
│   │   ├── apps/           # App lifecycle management
│   │   └── ...             # Other modules
│   └── migrations/         # SQL migrations
├── package.json
└── README.md
```

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Windows 10/11

### Installation

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Development

This is Task 01 - Project Setup completed. The following features are implemented:

✅ Tauri 2.0 project structure  
✅ SQLite database with migrations  
✅ Configuration management (TOML)  
✅ Module stubs for all future tasks  
✅ React frontend foundation

## Next Steps

- Task 02: Core Overlay Window
- Task 03: DirectX Hook DLL
- Task 04: Game Lifecycle Detector
- ...and more

## License

Proprietary
