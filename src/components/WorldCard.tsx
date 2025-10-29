import { Play, Square, Trash2, Users } from 'lucide-react';
import type { World } from '../types';

interface WorldCardProps {
  world: World;
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onDelete: (id: string) => void;
  onSelect: (id: string) => void;
}

export default function WorldCard({ world, onStart, onStop, onDelete, onSelect }: WorldCardProps) {
  const statusColor = {
    running: 'bg-green-500',
    stopped: 'bg-gray-500',
    starting: 'bg-yellow-500',
    stopping: 'bg-orange-500',
  }[world.status];

  const statusText = {
    running: 'Running',
    stopped: 'Stopped',
    starting: 'Starting...',
    stopping: 'Stopping...',
  }[world.status];

  return (
    <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <h3
            className="text-xl font-semibold text-gray-900 mb-2 cursor-pointer hover:text-blue-600 transition-colors"
            onClick={() => onSelect(world.id)}
          >
            {world.name}
          </h3>
          <div className="flex items-center gap-2 text-sm text-gray-600">
            <span className={`w-2 h-2 rounded-full ${statusColor}`}></span>
            <span>{statusText}</span>
          </div>
        </div>
        <button
          onClick={() => onDelete(world.id)}
          className="text-gray-400 hover:text-red-600 transition-colors"
          title="Delete World"
        >
          <Trash2 className="w-5 h-5" />
        </button>
      </div>

      <div className="space-y-2 mb-4">
        <div className="flex items-center justify-between text-sm">
          <span className="text-gray-600">Port</span>
          <span className="font-medium text-gray-900">{world.port}</span>
        </div>
        <div className="flex items-center justify-between text-sm">
          <span className="text-gray-600 flex items-center gap-1">
            <Users className="w-4 h-4" />
            Players
          </span>
          <span className="font-medium text-gray-900">
            {world.player_count} / {world.max_players}
          </span>
        </div>
      </div>

      <div className="flex gap-2">
        {world.status === 'stopped' && (
          <button
            onClick={() => onStart(world.id)}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors font-medium"
          >
            <Play className="w-4 h-4" />
            Start
          </button>
        )}
        {world.status === 'running' && (
          <button
            onClick={() => onStop(world.id)}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors font-medium"
          >
            <Square className="w-4 h-4" />
            Stop
          </button>
        )}
        {(world.status === 'starting' || world.status === 'stopping') && (
          <button
            disabled
            className="flex-1 px-4 py-2 bg-gray-300 text-gray-600 rounded-lg cursor-not-allowed font-medium"
          >
            {statusText}
          </button>
        )}
      </div>

      <div className="mt-4 pt-4 border-t border-gray-200">
        <div className="text-xs text-gray-500">
          Connect: <span className="font-mono text-gray-900">localhost:{world.port}</span>
        </div>
      </div>
    </div>
  );
}
