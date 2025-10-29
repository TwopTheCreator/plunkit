export interface World {
  id: string;
  name: string;
  status: 'running' | 'stopped' | 'starting' | 'stopping';
  port: number;
  max_players: number;
  player_count: number;
}

export interface Player {
  username: string;
  x: number;
  y: number;
  z: number;
}

export interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
}

export interface ServerStats {
  active_worlds: number;
  total_players: number;
  cpu_usage: number;
  memory_usage: number;
}

export interface CreateWorldRequest {
  name: string;
  max_players?: number;
  settings?: Record<string, unknown>;
}

export interface CreatePluginRequest {
  name: string;
  version: string;
  author: string;
  description: string;
  script: string;
}
