import type { World, Player, Plugin, ServerStats, CreateWorldRequest, CreatePluginRequest } from '../types';

const API_BASE = 'http://localhost:3001/api';

export async function listWorlds(): Promise<World[]> {
  const response = await fetch(`${API_BASE}/worlds`);
  if (!response.ok) throw new Error('Failed to fetch worlds');
  return response.json();
}

export async function createWorld(data: CreateWorldRequest): Promise<World> {
  const response = await fetch(`${API_BASE}/worlds`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create world');
  return response.json();
}

export async function getWorld(id: string): Promise<World> {
  const response = await fetch(`${API_BASE}/worlds/${id}`);
  if (!response.ok) throw new Error('Failed to fetch world');
  return response.json();
}

export async function deleteWorld(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/worlds/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete world');
}

export async function startWorld(id: string): Promise<World> {
  const response = await fetch(`${API_BASE}/worlds/${id}/start`, {
    method: 'POST',
  });
  if (!response.ok) throw new Error('Failed to start world');
  return response.json();
}

export async function stopWorld(id: string): Promise<World> {
  const response = await fetch(`${API_BASE}/worlds/${id}/stop`, {
    method: 'POST',
  });
  if (!response.ok) throw new Error('Failed to stop world');
  return response.json();
}

export async function listWorldPlayers(worldId: string): Promise<Player[]> {
  const response = await fetch(`${API_BASE}/worlds/${worldId}/players`);
  if (!response.ok) throw new Error('Failed to fetch players');
  return response.json();
}

export async function listPlugins(): Promise<Plugin[]> {
  const response = await fetch(`${API_BASE}/plugins`);
  if (!response.ok) throw new Error('Failed to fetch plugins');
  return response.json();
}

export async function createPlugin(data: CreatePluginRequest): Promise<Plugin> {
  const response = await fetch(`${API_BASE}/plugins`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create plugin');
  return response.json();
}

export async function deletePlugin(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/plugins/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete plugin');
}

export async function enablePlugin(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/plugins/${id}/enable`, {
    method: 'POST',
  });
  if (!response.ok) throw new Error('Failed to enable plugin');
}

export async function disablePlugin(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/plugins/${id}/disable`, {
    method: 'POST',
  });
  if (!response.ok) throw new Error('Failed to disable plugin');
}

export async function getStats(): Promise<ServerStats> {
  const response = await fetch(`${API_BASE}/stats`);
  if (!response.ok) throw new Error('Failed to fetch stats');
  return response.json();
}
