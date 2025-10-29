import { useEffect, useState } from 'react';
import { ArrowLeft, Users, Box } from 'lucide-react';
import type { World, Player } from '../types';
import { listWorldPlayers } from '../lib/api';

interface WorldDetailsProps {
  world: World;
  onBack: () => void;
}

export default function WorldDetails({ world, onBack }: WorldDetailsProps) {
  const [players, setPlayers] = useState<Player[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPlayers();
    const interval = setInterval(loadPlayers, 5000);
    return () => clearInterval(interval);
  }, [world.id]);

  const loadPlayers = async () => {
    try {
      const data = await listWorldPlayers(world.id);
      setPlayers(data);
    } catch (error) {
      console.error('Failed to load players:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-600 hover:text-gray-900 transition-colors font-medium"
      >
        <ArrowLeft className="w-5 h-5" />
        Back to Worlds
      </button>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <div className="flex items-start justify-between mb-6">
          <div>
            <h2 className="text-3xl font-bold text-gray-900 mb-2">{world.name}</h2>
            <div className="flex items-center gap-2 text-sm text-gray-600">
              <span className={`w-2 h-2 rounded-full ${world.status === 'running' ? 'bg-green-500' : 'bg-gray-500'}`}></span>
              <span className="capitalize">{world.status}</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-gray-50 rounded-lg p-4">
            <div className="text-sm text-gray-600 mb-1">Port</div>
            <div className="text-2xl font-bold text-gray-900">{world.port}</div>
          </div>
          <div className="bg-gray-50 rounded-lg p-4">
            <div className="text-sm text-gray-600 mb-1">Players</div>
            <div className="text-2xl font-bold text-gray-900">
              {world.player_count} / {world.max_players}
            </div>
          </div>
          <div className="bg-gray-50 rounded-lg p-4">
            <div className="text-sm text-gray-600 mb-1">Status</div>
            <div className="text-2xl font-bold text-gray-900 capitalize">{world.status}</div>
          </div>
          <div className="bg-gray-50 rounded-lg p-4">
            <div className="text-sm text-gray-600 mb-1">ID</div>
            <div className="text-xs font-mono text-gray-900 truncate">{world.id}</div>
          </div>
        </div>

        <div className="mt-6 pt-6 border-t border-gray-200">
          <div className="text-sm text-gray-600 mb-2">Connection String</div>
          <code className="block bg-gray-100 px-4 py-3 rounded-lg text-sm font-mono text-gray-900">
            localhost:{world.port}
          </code>
        </div>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <div className="flex items-center gap-2 mb-4">
          <Users className="w-5 h-5 text-gray-700" />
          <h3 className="text-xl font-bold text-gray-900">Online Players</h3>
        </div>

        {loading ? (
          <div className="text-center py-8 text-gray-500">Loading players...</div>
        ) : players.length === 0 ? (
          <div className="text-center py-8 text-gray-500">No players online</div>
        ) : (
          <div className="space-y-2">
            {players.map((player, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-700 rounded-lg flex items-center justify-center">
                    <span className="text-white font-bold text-sm">
                      {player.username.charAt(0).toUpperCase()}
                    </span>
                  </div>
                  <div>
                    <div className="font-medium text-gray-900">{player.username}</div>
                    <div className="text-xs text-gray-500">
                      Position: {player.x.toFixed(1)}, {player.y.toFixed(1)}, {player.z.toFixed(1)}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <div className="flex items-center gap-2 mb-4">
          <Box className="w-5 h-5 text-gray-700" />
          <h3 className="text-xl font-bold text-gray-900">World Information</h3>
        </div>

        <div className="space-y-3">
          <div className="flex justify-between py-2 border-b border-gray-100">
            <span className="text-gray-600">World Type</span>
            <span className="font-medium text-gray-900">Flat</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-100">
            <span className="text-gray-600">Sandbox Type</span>
            <span className="font-medium text-gray-900">WASM</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-100">
            <span className="text-gray-600">Max Players</span>
            <span className="font-medium text-gray-900">{world.max_players}</span>
          </div>
          <div className="flex justify-between py-2">
            <span className="text-gray-600">Render Distance</span>
            <span className="font-medium text-gray-900">10 chunks</span>
          </div>
        </div>
      </div>
    </div>
  );
}
