import { useState, useEffect } from 'react';
import { Globe, Code, LayoutDashboard } from 'lucide-react';
import WorldCard from './components/WorldCard';
import CreateWorldModal from './components/CreateWorldModal';
import WorldDetails from './components/WorldDetails';
import PluginManager from './components/PluginManager';
import ServerStats from './components/ServerStats';
import type { World } from './types';
import { listWorlds, createWorld, deleteWorld, startWorld, stopWorld } from './lib/api';

type View = 'dashboard' | 'worlds' | 'plugins' | 'world-details';

function App() {
  const [view, setView] = useState<View>('dashboard');
  const [worlds, setWorlds] = useState<World[]>([]);
  const [selectedWorld, setSelectedWorld] = useState<World | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadWorlds();
    const interval = setInterval(loadWorlds, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadWorlds = async () => {
    try {
      const data = await listWorlds();
      setWorlds(data);
    } catch (error) {
      console.error('Failed to load worlds:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateWorld = async (name: string, maxPlayers: number) => {
    try {
      await createWorld({ name, max_players: maxPlayers });
      await loadWorlds();
    } catch (error) {
      console.error('Failed to create world:', error);
      alert('Failed to create world');
    }
  };

  const handleDeleteWorld = async (id: string) => {
    if (!confirm('Are you sure you want to delete this world? This action cannot be undone.')) {
      return;
    }
    try {
      await deleteWorld(id);
      await loadWorlds();
      if (selectedWorld?.id === id) {
        setSelectedWorld(null);
        setView('worlds');
      }
    } catch (error) {
      console.error('Failed to delete world:', error);
      alert('Failed to delete world');
    }
  };

  const handleStartWorld = async (id: string) => {
    try {
      await startWorld(id);
      await loadWorlds();
    } catch (error) {
      console.error('Failed to start world:', error);
      alert('Failed to start world');
    }
  };

  const handleStopWorld = async (id: string) => {
    try {
      await stopWorld(id);
      await loadWorlds();
    } catch (error) {
      console.error('Failed to stop world:', error);
      alert('Failed to stop world');
    }
  };

  const handleSelectWorld = (id: string) => {
    const world = worlds.find((w) => w.id === id);
    if (world) {
      setSelectedWorld(world);
      setView('world-details');
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100">
      <nav className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-gradient-to-br from-blue-600 to-blue-700 rounded-lg flex items-center justify-center">
                <Globe className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-gray-900">Plunkit</h1>
                <p className="text-xs text-gray-600">Minecraft WASM Sandbox Server</p>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={() => setView('dashboard')}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all font-medium ${
                  view === 'dashboard'
                    ? 'bg-blue-600 text-white shadow-md'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <LayoutDashboard className="w-4 h-4" />
                Dashboard
              </button>
              <button
                onClick={() => setView('worlds')}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all font-medium ${
                  view === 'worlds' || view === 'world-details'
                    ? 'bg-blue-600 text-white shadow-md'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <Globe className="w-4 h-4" />
                Worlds
              </button>
              <button
                onClick={() => setView('plugins')}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all font-medium ${
                  view === 'plugins'
                    ? 'bg-blue-600 text-white shadow-md'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <Code className="w-4 h-4" />
                Plugins
              </button>
            </div>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {view === 'dashboard' && (
          <div className="space-y-8">
            <div>
              <h2 className="text-3xl font-bold text-gray-900 mb-2">Dashboard</h2>
              <p className="text-gray-600">Monitor your Plunkit server infrastructure</p>
            </div>

            <ServerStats />

            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-8">
              <h3 className="text-xl font-bold text-gray-900 mb-4">About Plunkit</h3>
              <div className="space-y-4 text-gray-700">
                <p>
                  Plunkit is a revolutionary Minecraft Java server platform that runs worlds in isolated
                  WASM sandboxes. Each world operates independently with full security and performance isolation.
                </p>
                <div className="grid md:grid-cols-3 gap-6 mt-6">
                  <div className="bg-gradient-to-br from-blue-50 to-blue-100 rounded-lg p-6">
                    <h4 className="font-bold text-gray-900 mb-2">WASM Sandboxing</h4>
                    <p className="text-sm text-gray-700">
                      Each world runs in its own WebAssembly sandbox for complete isolation and security.
                    </p>
                  </div>
                  <div className="bg-gradient-to-br from-green-50 to-green-100 rounded-lg p-6">
                    <h4 className="font-bold text-gray-900 mb-2">Lua Plugins</h4>
                    <p className="text-sm text-gray-700">
                      Extend functionality with powerful Lua plugins that hook into game events.
                    </p>
                  </div>
                  <div className="bg-gradient-to-br from-orange-50 to-orange-100 rounded-lg p-6">
                    <h4 className="font-bold text-gray-900 mb-2">ECS Architecture</h4>
                    <p className="text-sm text-gray-700">
                      Built on Bevy ECS for high-performance entity and world management.
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-xl font-bold text-gray-900">Quick Actions</h3>
              </div>
              <div className="grid md:grid-cols-2 gap-4">
                <button
                  onClick={() => {
                    setView('worlds');
                    setShowCreateModal(true);
                  }}
                  className="flex items-center gap-3 p-4 bg-gradient-to-r from-blue-600 to-blue-700 text-white rounded-lg hover:from-blue-700 hover:to-blue-800 transition-all shadow-md"
                >
                  <Globe className="w-6 h-6" />
                  <div className="text-left">
                    <div className="font-bold">Create New World</div>
                    <div className="text-sm text-blue-100">Start a new Minecraft server</div>
                  </div>
                </button>
                <button
                  onClick={() => setView('plugins')}
                  className="flex items-center gap-3 p-4 bg-gradient-to-r from-green-600 to-green-700 text-white rounded-lg hover:from-green-700 hover:to-green-800 transition-all shadow-md"
                >
                  <Code className="w-6 h-6" />
                  <div className="text-left">
                    <div className="font-bold">Manage Plugins</div>
                    <div className="text-sm text-green-100">Create and configure Lua plugins</div>
                  </div>
                </button>
              </div>
            </div>
          </div>
        )}

        {view === 'worlds' && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-3xl font-bold text-gray-900">Minecraft Worlds</h2>
                <p className="text-gray-600 mt-1">Manage your sandboxed server instances</p>
              </div>
              <button
                onClick={() => setShowCreateModal(true)}
                className="flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-all shadow-md font-medium"
              >
                <Globe className="w-5 h-5" />
                Create World
              </button>
            </div>

            {loading ? (
              <div className="text-center py-12 text-gray-500">Loading worlds...</div>
            ) : worlds.length === 0 ? (
              <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-12 text-center">
                <Globe className="w-16 h-16 text-gray-300 mx-auto mb-4" />
                <h3 className="text-xl font-semibold text-gray-900 mb-2">No Worlds Yet</h3>
                <p className="text-gray-600 mb-6">
                  Create your first Minecraft world to get started with Plunkit
                </p>
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="inline-flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                >
                  <Globe className="w-5 h-5" />
                  Create World
                </button>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {worlds.map((world) => (
                  <WorldCard
                    key={world.id}
                    world={world}
                    onStart={handleStartWorld}
                    onStop={handleStopWorld}
                    onDelete={handleDeleteWorld}
                    onSelect={handleSelectWorld}
                  />
                ))}
              </div>
            )}
          </div>
        )}

        {view === 'world-details' && selectedWorld && (
          <WorldDetails world={selectedWorld} onBack={() => setView('worlds')} />
        )}

        {view === 'plugins' && <PluginManager />}
      </main>

      {showCreateModal && (
        <CreateWorldModal
          onClose={() => setShowCreateModal(false)}
          onCreate={handleCreateWorld}
        />
      )}
    </div>
  );
}

export default App;
