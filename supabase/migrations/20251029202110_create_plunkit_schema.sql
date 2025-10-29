/*
  # Plunkit Server Database Schema

  ## Overview
  Complete database schema for Plunkit - a Minecraft Java server WASM sandbox system.

  ## 1. New Tables
  
  ### `worlds`
  Stores all Minecraft world instances running in WASM sandboxes.
  - `id` (uuid, primary key) - Unique world identifier
  - `name` (text) - Display name of the world
  - `owner_id` (uuid) - Reference to auth.users who created this world
  - `status` (text) - World status: 'running', 'stopped', 'starting', 'stopping'
  - `port` (integer) - Port number for this world's Minecraft server
  - `max_players` (integer) - Maximum number of concurrent players
  - `wasm_module_url` (text, nullable) - URL to custom WASM module if used
  - `settings` (jsonb) - World settings (gamemode, difficulty, spawn, etc.)
  - `created_at` (timestamptz) - Creation timestamp
  - `updated_at` (timestamptz) - Last update timestamp

  ### `players`
  Tracks player sessions and data across all worlds.
  - `id` (uuid, primary key) - Unique player identifier
  - `username` (text, unique) - Minecraft username
  - `uuid` (uuid, unique) - Minecraft UUID
  - `last_world_id` (uuid, nullable) - Last world they connected to
  - `total_playtime` (integer) - Total playtime in seconds
  - `created_at` (timestamptz) - First connection timestamp
  - `last_seen` (timestamptz) - Last connection timestamp

  ### `world_players`
  Junction table for active player sessions in worlds.
  - `id` (uuid, primary key)
  - `world_id` (uuid) - Reference to worlds
  - `player_id` (uuid) - Reference to players
  - `position_x` (double precision) - Current X position
  - `position_y` (double precision) - Current Y position
  - `position_z` (double precision) - Current Z position
  - `health` (real) - Player health
  - `gamemode` (integer) - Player gamemode
  - `connected_at` (timestamptz) - When player joined this session
  - `last_activity` (timestamptz) - Last activity timestamp

  ### `chunks`
  Stores chunk data for persistence.
  - `id` (uuid, primary key)
  - `world_id` (uuid) - Reference to worlds
  - `chunk_x` (integer) - Chunk X coordinate
  - `chunk_z` (integer) - Chunk Z coordinate
  - `data` (bytea) - Compressed chunk data
  - `generated_at` (timestamptz) - Generation timestamp
  - `modified_at` (timestamptz) - Last modification timestamp

  ### `plugins`
  Lua plugin definitions.
  - `id` (uuid, primary key)
  - `name` (text, unique) - Plugin name
  - `version` (text) - Plugin version
  - `author` (text) - Plugin author
  - `description` (text) - Plugin description
  - `script` (text) - Lua script source code
  - `enabled` (boolean) - Whether plugin is enabled globally
  - `created_at` (timestamptz) - Creation timestamp
  - `updated_at` (timestamptz) - Last update timestamp

  ### `world_plugins`
  Junction table for which plugins are active in which worlds.
  - `id` (uuid, primary key)
  - `world_id` (uuid) - Reference to worlds
  - `plugin_id` (uuid) - Reference to plugins
  - `enabled` (boolean) - Whether plugin is enabled for this world
  - `config` (jsonb, nullable) - Plugin-specific configuration

  ### `server_stats`
  System metrics and statistics.
  - `id` (uuid, primary key)
  - `timestamp` (timestamptz) - Measurement timestamp
  - `active_worlds` (integer) - Number of active worlds
  - `total_players` (integer) - Total connected players
  - `cpu_usage` (real) - CPU usage percentage
  - `memory_usage` (bigint) - Memory usage in bytes
  - `network_in` (bigint) - Network bytes received
  - `network_out` (bigint) - Network bytes sent

  ## 2. Security (Row Level Security)
  
  All tables have RLS enabled with appropriate policies:
  - World owners can manage their own worlds
  - Players can read their own data
  - Authenticated users can view public world information
  - Only authenticated users can create worlds
  - Stats are readable by all authenticated users

  ## 3. Indexes
  
  Performance indexes on:
  - Foreign key relationships
  - Frequently queried fields (world status, player username)
  - Chunk coordinates for spatial queries

  ## 4. Important Notes
  
  - All timestamps use timestamptz for proper timezone handling
  - JSONB used for flexible settings and configuration storage
  - Bytea for efficient binary chunk data storage
  - Unique constraints prevent duplicate usernames and UUIDs
  - Foreign keys ensure referential integrity
  - Default values set for boolean and status fields
*/

CREATE TABLE IF NOT EXISTS worlds (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text NOT NULL,
  owner_id uuid REFERENCES auth.users(id) ON DELETE CASCADE,
  status text NOT NULL DEFAULT 'stopped',
  port integer NOT NULL DEFAULT 25565,
  max_players integer NOT NULL DEFAULT 20,
  wasm_module_url text,
  settings jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS players (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  username text UNIQUE NOT NULL,
  uuid uuid UNIQUE NOT NULL,
  last_world_id uuid REFERENCES worlds(id) ON DELETE SET NULL,
  total_playtime integer NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  last_seen timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS world_players (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  world_id uuid NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
  player_id uuid NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  position_x double precision NOT NULL DEFAULT 0,
  position_y double precision NOT NULL DEFAULT 64,
  position_z double precision NOT NULL DEFAULT 0,
  health real NOT NULL DEFAULT 20,
  gamemode integer NOT NULL DEFAULT 0,
  connected_at timestamptz NOT NULL DEFAULT now(),
  last_activity timestamptz NOT NULL DEFAULT now(),
  UNIQUE(world_id, player_id)
);

CREATE TABLE IF NOT EXISTS chunks (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  world_id uuid NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
  chunk_x integer NOT NULL,
  chunk_z integer NOT NULL,
  data bytea NOT NULL,
  generated_at timestamptz NOT NULL DEFAULT now(),
  modified_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(world_id, chunk_x, chunk_z)
);

CREATE TABLE IF NOT EXISTS plugins (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text UNIQUE NOT NULL,
  version text NOT NULL DEFAULT '1.0.0',
  author text NOT NULL DEFAULT 'Unknown',
  description text NOT NULL DEFAULT '',
  script text NOT NULL,
  enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS world_plugins (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  world_id uuid NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
  plugin_id uuid NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  enabled boolean NOT NULL DEFAULT true,
  config jsonb DEFAULT '{}'::jsonb,
  UNIQUE(world_id, plugin_id)
);

CREATE TABLE IF NOT EXISTS server_stats (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  timestamp timestamptz NOT NULL DEFAULT now(),
  active_worlds integer NOT NULL DEFAULT 0,
  total_players integer NOT NULL DEFAULT 0,
  cpu_usage real NOT NULL DEFAULT 0,
  memory_usage bigint NOT NULL DEFAULT 0,
  network_in bigint NOT NULL DEFAULT 0,
  network_out bigint NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_worlds_owner ON worlds(owner_id);
CREATE INDEX IF NOT EXISTS idx_worlds_status ON worlds(status);
CREATE INDEX IF NOT EXISTS idx_players_username ON players(username);
CREATE INDEX IF NOT EXISTS idx_players_uuid ON players(uuid);
CREATE INDEX IF NOT EXISTS idx_world_players_world ON world_players(world_id);
CREATE INDEX IF NOT EXISTS idx_world_players_player ON world_players(player_id);
CREATE INDEX IF NOT EXISTS idx_chunks_world ON chunks(world_id);
CREATE INDEX IF NOT EXISTS idx_chunks_coords ON chunks(world_id, chunk_x, chunk_z);
CREATE INDEX IF NOT EXISTS idx_world_plugins_world ON world_plugins(world_id);
CREATE INDEX IF NOT EXISTS idx_server_stats_timestamp ON server_stats(timestamp DESC);

ALTER TABLE worlds ENABLE ROW LEVEL SECURITY;
ALTER TABLE players ENABLE ROW LEVEL SECURITY;
ALTER TABLE world_players ENABLE ROW LEVEL SECURITY;
ALTER TABLE chunks ENABLE ROW LEVEL SECURITY;
ALTER TABLE plugins ENABLE ROW LEVEL SECURITY;
ALTER TABLE world_plugins ENABLE ROW LEVEL SECURITY;
ALTER TABLE server_stats ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view all worlds"
  ON worlds FOR SELECT
  TO authenticated
  USING (true);

CREATE POLICY "Users can create their own worlds"
  ON worlds FOR INSERT
  TO authenticated
  WITH CHECK (auth.uid() = owner_id);

CREATE POLICY "Users can update their own worlds"
  ON worlds FOR UPDATE
  TO authenticated
  USING (auth.uid() = owner_id)
  WITH CHECK (auth.uid() = owner_id);

CREATE POLICY "Users can delete their own worlds"
  ON worlds FOR DELETE
  TO authenticated
  USING (auth.uid() = owner_id);

CREATE POLICY "Anyone can view players"
  ON players FOR SELECT
  TO authenticated
  USING (true);

CREATE POLICY "System can manage players"
  ON players FOR ALL
  TO authenticated
  USING (true)
  WITH CHECK (true);

CREATE POLICY "Anyone can view world players"
  ON world_players FOR SELECT
  TO authenticated
  USING (true);

CREATE POLICY "System can manage world players"
  ON world_players FOR ALL
  TO authenticated
  USING (true)
  WITH CHECK (true);

CREATE POLICY "World owners can view their chunks"
  ON chunks FOR SELECT
  TO authenticated
  USING (
    EXISTS (
      SELECT 1 FROM worlds
      WHERE worlds.id = chunks.world_id
      AND worlds.owner_id = auth.uid()
    )
  );

CREATE POLICY "World owners can manage their chunks"
  ON chunks FOR ALL
  TO authenticated
  USING (
    EXISTS (
      SELECT 1 FROM worlds
      WHERE worlds.id = chunks.world_id
      AND worlds.owner_id = auth.uid()
    )
  )
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM worlds
      WHERE worlds.id = chunks.world_id
      AND worlds.owner_id = auth.uid()
    )
  );

CREATE POLICY "Anyone can view plugins"
  ON plugins FOR SELECT
  TO authenticated
  USING (true);

CREATE POLICY "Authenticated users can create plugins"
  ON plugins FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Authenticated users can update plugins"
  ON plugins FOR UPDATE
  TO authenticated
  USING (true)
  WITH CHECK (true);

CREATE POLICY "Authenticated users can delete plugins"
  ON plugins FOR DELETE
  TO authenticated
  USING (true);

CREATE POLICY "World owners can manage their world plugins"
  ON world_plugins FOR ALL
  TO authenticated
  USING (
    EXISTS (
      SELECT 1 FROM worlds
      WHERE worlds.id = world_plugins.world_id
      AND worlds.owner_id = auth.uid()
    )
  )
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM worlds
      WHERE worlds.id = world_plugins.world_id
      AND worlds.owner_id = auth.uid()
    )
  );

CREATE POLICY "Anyone can view server stats"
  ON server_stats FOR SELECT
  TO authenticated
  USING (true);

CREATE POLICY "System can insert server stats"
  ON server_stats FOR INSERT
  TO authenticated
  WITH CHECK (true);
