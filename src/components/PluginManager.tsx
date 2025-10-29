import { useEffect, useState } from 'react';
import { Plus, Code, Trash2, Power, PowerOff } from 'lucide-react';
import type { Plugin } from '../types';
import { listPlugins, createPlugin, deletePlugin, enablePlugin, disablePlugin } from '../lib/api';

export default function PluginManager() {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);

  useEffect(() => {
    loadPlugins();
  }, []);

  const loadPlugins = async () => {
    try {
      const data = await listPlugins();
      setPlugins(data);
    } catch (error) {
      console.error('Failed to load plugins:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreatePlugin = async (
    name: string,
    version: string,
    author: string,
    description: string,
    script: string
  ) => {
    try {
      await createPlugin({ name, version, author, description, script });
      await loadPlugins();
    } catch (error) {
      console.error('Failed to create plugin:', error);
      alert('Failed to create plugin');
    }
  };

  const handleDeletePlugin = async (id: string) => {
    if (!confirm('Are you sure you want to delete this plugin?')) return;
    try {
      await deletePlugin(id);
      await loadPlugins();
    } catch (error) {
      console.error('Failed to delete plugin:', error);
    }
  };

  const handleTogglePlugin = async (id: string, enabled: boolean) => {
    try {
      if (enabled) {
        await disablePlugin(id);
      } else {
        await enablePlugin(id);
      }
      await loadPlugins();
    } catch (error) {
      console.error('Failed to toggle plugin:', error);
    }
  };

  if (loading) {
    return <div className="text-center py-12 text-gray-500">Loading plugins...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900">Plugin Manager</h2>
          <p className="text-gray-600 mt-1">Manage Lua plugins for your worlds</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
        >
          <Plus className="w-5 h-5" />
          Create Plugin
        </button>
      </div>

      {plugins.length === 0 ? (
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-12 text-center">
          <Code className="w-16 h-16 text-gray-300 mx-auto mb-4" />
          <h3 className="text-xl font-semibold text-gray-900 mb-2">No Plugins Yet</h3>
          <p className="text-gray-600 mb-6">
            Create your first Lua plugin to extend server functionality
          </p>
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
          >
            <Plus className="w-5 h-5" />
            Create Plugin
          </button>
        </div>
      ) : (
        <div className="grid gap-4">
          {plugins.map((plugin) => (
            <div
              key={plugin.id}
              className="bg-white rounded-xl shadow-sm border border-gray-200 p-6"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <h3 className="text-xl font-semibold text-gray-900">{plugin.name}</h3>
                    <span className="text-sm text-gray-500">v{plugin.version}</span>
                    {plugin.enabled ? (
                      <span className="px-2 py-1 bg-green-100 text-green-700 text-xs font-medium rounded-full">
                        Enabled
                      </span>
                    ) : (
                      <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs font-medium rounded-full">
                        Disabled
                      </span>
                    )}
                  </div>
                  <p className="text-gray-600 mb-3">{plugin.description}</p>
                  <p className="text-sm text-gray-500">by {plugin.author}</p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleTogglePlugin(plugin.id, plugin.enabled)}
                    className={`p-2 rounded-lg transition-colors ${
                      plugin.enabled
                        ? 'text-gray-600 hover:text-orange-600 hover:bg-orange-50'
                        : 'text-gray-600 hover:text-green-600 hover:bg-green-50'
                    }`}
                    title={plugin.enabled ? 'Disable' : 'Enable'}
                  >
                    {plugin.enabled ? (
                      <PowerOff className="w-5 h-5" />
                    ) : (
                      <Power className="w-5 h-5" />
                    )}
                  </button>
                  <button
                    onClick={() => handleDeletePlugin(plugin.id)}
                    className="p-2 text-gray-600 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                    title="Delete"
                  >
                    <Trash2 className="w-5 h-5" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {showCreateModal && (
        <CreatePluginModal
          onClose={() => setShowCreateModal(false)}
          onCreate={handleCreatePlugin}
        />
      )}
    </div>
  );
}

interface CreatePluginModalProps {
  onClose: () => void;
  onCreate: (name: string, version: string, author: string, description: string, script: string) => void;
}

function CreatePluginModal({ onClose, onCreate }: CreatePluginModalProps) {
  const [name, setName] = useState('');
  const [version, setVersion] = useState('1.0.0');
  const [author, setAuthor] = useState('');
  const [description, setDescription] = useState('');
  const [script, setScript] = useState(defaultScript);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim() && author.trim() && script.trim()) {
      onCreate(name.trim(), version.trim(), author.trim(), description.trim(), script.trim());
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
        <div className="sticky top-0 bg-white border-b border-gray-200 p-6 z-10">
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-bold text-gray-900">Create New Plugin</h2>
            <button onClick={onClose} className="text-gray-400 hover:text-gray-600 transition-colors">
              <Plus className="w-6 h-6 rotate-45" />
            </button>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Plugin Name</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="my-plugin"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Version</label>
              <input
                type="text"
                value={version}
                onChange={(e) => setVersion(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="1.0.0"
                required
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Author</label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Your Name"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Description</label>
            <input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="A brief description of your plugin"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Lua Script</label>
            <textarea
              value={script}
              onChange={(e) => setScript(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm h-64"
              required
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
            >
              Create Plugin
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

const defaultScript = `-- Example plugin script
function on_enable()
  plunkit.log("Plugin enabled!")
end

function on_disable()
  plunkit.log("Plugin disabled!")
end

function on_player_join(player_name)
  plunkit.log("Player joined: " .. player_name)
  plunkit.broadcast("Welcome " .. player_name .. " to the server!")
  return false
end

function on_player_chat(player_name, message)
  plunkit.log(player_name .. " says: " .. message)
  return false
end

function on_block_break(player_name, x, y, z)
  plunkit.log(player_name .. " broke block at " .. x .. ", " .. y .. ", " .. z)
  return false
end

function on_block_place(player_name, x, y, z, block_id)
  plunkit.log(player_name .. " placed block " .. block_id .. " at " .. x .. ", " .. y .. ", " .. z)
  return false
end`;
