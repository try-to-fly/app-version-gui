import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, CacheConfig } from "@/types/software";
import { DEFAULT_NOTIFICATION_CONFIG } from "@/types/software";

interface SettingsState {
  settings: AppSettings;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  updateCacheConfig: (config: Partial<CacheConfig>) => void;
  setGithubToken: (token: string | undefined) => void;
  clearCache: () => Promise<void>;
}

const defaultSettings: AppSettings = {
  cache: {
    ttlMinutes: 30,
    autoRefreshEnabled: true,
    autoRefreshInterval: 60,
  },
  githubToken: undefined,
  theme: "system",
  notification: DEFAULT_NOTIFICATION_CONFIG,
};

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: defaultSettings,
  isLoading: false,
  error: null,

  fetchSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await invoke<AppSettings>("get_settings");
      set({ settings, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  saveSettings: async (settings) => {
    try {
      await invoke("save_settings", { newSettings: settings });
      // 更新后端调度器
      await invoke("update_scheduler", {
        enabled: settings.cache.autoRefreshEnabled,
        intervalMinutes: settings.cache.autoRefreshInterval,
      });
      set({ settings });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateCacheConfig: (config) => {
    set((state) => ({
      settings: {
        ...state.settings,
        cache: { ...state.settings.cache, ...config },
      },
    }));
  },

  setGithubToken: (token) => {
    set((state) => ({
      settings: { ...state.settings, githubToken: token },
    }));
  },

  clearCache: async () => {
    try {
      await invoke("clear_cache");
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
