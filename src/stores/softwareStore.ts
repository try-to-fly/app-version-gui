import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  Software,
  SoftwareFormData,
  VersionCheckResult,
} from "@/types/software";

interface SoftwareState {
  softwares: Software[];
  isLoading: boolean;
  isChecking: boolean;
  error: string | null;

  // Actions
  fetchSoftwares: () => Promise<void>;
  addSoftware: (form: SoftwareFormData) => Promise<Software>;
  updateSoftware: (id: string, form: SoftwareFormData) => Promise<Software>;
  deleteSoftware: (id: string) => Promise<void>;
  toggleSoftware: (id: string, enabled: boolean) => Promise<void>;
  checkVersion: (id: string, forceRefresh?: boolean) => Promise<VersionCheckResult>;
  checkAllVersions: () => Promise<VersionCheckResult[]>;
  clearError: () => void;
}

export const useSoftwareStore = create<SoftwareState>((set) => ({
  softwares: [],
  isLoading: false,
  isChecking: false,
  error: null,

  fetchSoftwares: async () => {
    set({ isLoading: true, error: null });
    try {
      const softwares = await invoke<Software[]>("get_all_softwares");
      set({ softwares, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  addSoftware: async (form) => {
    try {
      const software = await invoke<Software>("add_software", { form });
      set((state) => ({ softwares: [...state.softwares, software] }));
      return software;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateSoftware: async (id, form) => {
    try {
      const software = await invoke<Software>("update_software", { id, form });
      set((state) => ({
        softwares: state.softwares.map((s) => (s.id === id ? software : s)),
      }));
      return software;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteSoftware: async (id) => {
    try {
      await invoke("delete_software", { id });
      set((state) => ({
        softwares: state.softwares.filter((s) => s.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  toggleSoftware: async (id, enabled) => {
    try {
      await invoke("toggle_software", { id, enabled });
      set((state) => ({
        softwares: state.softwares.map((s) =>
          s.id === id ? { ...s, enabled } : s
        ),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  checkVersion: async (id, forceRefresh = false) => {
    set({ isChecking: true });
    try {
      const result = await invoke<VersionCheckResult>("check_version", {
        id,
        forceRefresh,
      });

      // Update local state with the result
      set((state) => ({
        softwares: state.softwares.map((s) =>
          s.id === id
            ? {
                ...s,
                latestVersion: result.latestVersion,
                localVersion: result.localVersion,
                publishedAt: result.publishedAt,
                lastCheckedAt: new Date().toISOString(),
              }
            : s
        ),
        isChecking: false,
      }));

      return result;
    } catch (error) {
      set({ error: String(error), isChecking: false });
      throw error;
    }
  },

  checkAllVersions: async () => {
    set({ isChecking: true });
    try {
      const results = await invoke<VersionCheckResult[]>("check_all_versions");

      // Update local state with the results
      set((state) => {
        const updatedSoftwares = state.softwares.map((s) => {
          const result = results.find((r) => r.softwareId === s.id);
          if (result) {
            return {
              ...s,
              latestVersion: result.latestVersion,
              localVersion: result.localVersion,
              publishedAt: result.publishedAt,
              lastCheckedAt: new Date().toISOString(),
            };
          }
          return s;
        });
        return { softwares: updatedSoftwares, isChecking: false };
      });

      return results;
    } catch (error) {
      set({ error: String(error), isChecking: false });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));
