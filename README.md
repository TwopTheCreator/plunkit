# Plunkit

A revolutionary Minecraft Java server platform that runs worlds in isolated WASM sandboxes. Each world operates independently with full security and performance isolation.

## Features

### WASM Sandboxing
- Each Minecraft world runs in its own WebAssembly sandbox using Wasmtime
- Complete isolation and security between worlds
- Resource limiting (CPU, memory) per sandbox
- Host function API for world interaction

### Minecraft Protocol Support
- Full Minecraft Java Edition protocol implementation
- Supports handshake, status, login, and play states
- Chunk generation and streaming
- Player position and rotation tracking
- Block break/place events
- Chat messages

### Lua Plugin System
- Powerful Lua scripting engine using mlua
- Event hooks: player join, chat, block break/place
- Plugin API for world manipulation
- Hot-reload plugin support
- Example plugins included

### ECS World Management
- Built on Bevy ECS for high-performance entity management
- Efficient chunk storage and loading
- Player entity system
- Component-based architecture

### Web Dashboard
- React-based management interface
- Create and manage multiple worlds
- Real-time player monitoring
- Server statistics and metrics
- Plugin management UI

### Database Persistence
- Supabase integration for data storage
- World state persistence
- Player data tracking
- Chunk storage
- Plugin configuration

## Architecture

### Backend (Rust)
- `plunkit_server/src/protocol/` - Minecraft protocol implementation
- `plunkit_server/src/sandbox/` - WASM runtime and sandbox manager
- `plunkit_server/src/lua/` - Lua scripting engine and plugin system
- `plunkit_server/src/world/` - ECS world management and chunks
- `plunkit_server/src/server/` - Minecraft server and player sessions
- `plunkit_server/src/api/` - Axum REST API for web dashboard
- `plunkit_server/src/db/` - Database integration layer

### Frontend (React + TypeScript)
- Dashboard for monitoring server stats
- World creation and management
- Player tracking and visualization
- Plugin manager with code editor
- Real-time updates via polling

## Running the Project

### Prerequisites
- Rust (stable toolchain)
- Node.js and npm
- Supabase account (environment variables configured)

### Backend Setup
```bash
cd plunkit_server
cargo build --release
cargo run
```

The Rust server will start on `http://localhost:3001`

### Frontend Setup
```bash
npm install
npm run dev
```

The React dashboard will be available at `http://localhost:5173`

### Database Setup
Database migrations are automatically applied. Tables include:
- `worlds` - Minecraft world instances
- `players` - Player data and statistics
- `world_players` - Active player sessions
- `chunks` - Chunk data storage
- `plugins` - Lua plugin definitions
- `world_plugins` - Plugin assignments
- `server_stats` - System metrics

## Example Lua Plugins

### Welcome Plugin
Broadcasts welcome messages when players join.

### Protection Plugin
Prevents breaking protected blocks (bedrock, etc).

### Commands Plugin
Adds custom commands: /help, /spawn, /players, /time, /give

### Autosave Plugin
Automatically saves world data at regular intervals.

### Events Plugin
Tracks player statistics (blocks broken/placed).

## Creating Custom Plugins

Create a Lua script with these hooks:

```lua
function on_enable()
  plunkit.log("Plugin enabled!")
end

function on_disable()
  plunkit.log("Plugin disabled!")
end

function on_player_join(player_name)
  plunkit.broadcast("Welcome " .. player_name)
  return false -- return true to cancel event
end

function on_player_chat(player_name, message)
  plunkit.log(player_name .. " says: " .. message)
  return false
end

function on_block_break(player_name, x, y, z)
  -- return true to prevent breaking
  return false
end

function on_block_place(player_name, x, y, z, block_id)
  -- return true to prevent placing
  return false
end
```

### Plugin API

Available functions:
- `plunkit.log(message)` - Log to server console
- `plunkit.broadcast(message)` - Send message to all players
- `plunkit.get_block(x, y, z)` - Get block ID at position
- `plunkit.set_block(x, y, z, block_id)` - Set block at position
- `plunkit.spawn_entity(type, x, y, z)` - Spawn entity
- `plunkit.get_players()` - Get list of online players
- `plunkit.teleport_player(name, x, y, z)` - Teleport player
- `plunkit.give_item(name, item, count)` - Give item to player

## Connecting with Minecraft Client

1. Create a world in the dashboard
2. Start the world
3. Note the port (default: 25565)
4. In Minecraft, add server: `localhost:25565`
5. Connect and play!

## Technology Stack

### Backend
- Rust (Tokio async runtime)
- Wasmtime (WASM sandbox)
- mlua (Lua scripting)
- Bevy ECS (entity system)
- Axum (web framework)
- Supabase (database)

### Frontend
- React 18
- TypeScript
- Tailwind CSS
- Lucide Icons
- Vite (build tool)

## Security Features

- WASM sandboxing prevents malicious world code
- Resource limits per sandbox (memory, CPU)
- Row Level Security on database tables
- Authentication required for world management
- Plugin execution isolation

## Performance

- Async/await throughout for non-blocking I/O
- ECS architecture for efficient entity updates
- Chunk streaming to reduce memory usage
- WASM JIT compilation for fast execution
- Connection pooling and caching

## Future Enhancements

- Multi-world load balancing
- Redis caching layer
- WebSocket for real-time updates
- Custom world generation algorithms
- NBT data serialization
- Inventory management
- Advanced entity AI
- Redstone simulation
- More protocol features (combat, trading, etc)
