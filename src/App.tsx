import { useEffect, useState, useCallback, useRef } from "react";
import { Header } from "@/components/layout/Header";
import { SoftwareTable } from "@/components/software/SoftwareTable";
import { AddSoftwareDialog } from "@/components/software/AddSoftwareDialog";
import { EditSoftwareDialog } from "@/components/software/EditSoftwareDialog";
import { SettingsDialog } from "@/components/settings/SettingsDialog";
import { Toaster, toast } from "@/components/ui/sonner";
import { useSoftwareStore } from "@/stores/softwareStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { fromNow } from "@/lib/time";
import type { Software, SoftwareFormData } from "@/types/software";

function App() {
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [settingsDialogOpen, setSettingsDialogOpen] = useState(false);
  const [editingSoftware, setEditingSoftware] = useState<Software | null>(null);
  const [refreshingId, setRefreshingId] = useState<string | null>(null);

  const {
    softwares,
    isLoading,
    isChecking,
    fetchSoftwares,
    addSoftware,
    updateSoftware,
    deleteSoftware,
    checkVersion,
    checkAllVersions,
  } = useSoftwareStore();

  const {
    settings,
    fetchSettings,
    saveSettings,
    clearCache,
  } = useSettingsStore();

  const autoRefreshRef = useRef<number | null>(null);

  // Apply theme mode
  useEffect(() => {
    const applyTheme = (mode: "light" | "dark") => {
      document.documentElement.classList.remove("light", "dark");
      document.documentElement.classList.add(mode);
    };

    if (settings.theme === "system") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handleChange = (e: MediaQueryListEvent | MediaQueryList) => {
        applyTheme(e.matches ? "dark" : "light");
      };
      handleChange(mediaQuery);
      mediaQuery.addEventListener("change", handleChange);
      return () => mediaQuery.removeEventListener("change", handleChange);
    } else {
      applyTheme(settings.theme);
    }
  }, [settings.theme]);

  // Initial data fetch
  useEffect(() => {
    fetchSoftwares();
    fetchSettings();
  }, [fetchSoftwares, fetchSettings]);

  // Auto refresh
  useEffect(() => {
    if (settings.cache.autoRefreshEnabled && settings.cache.autoRefreshInterval > 0) {
      const interval = settings.cache.autoRefreshInterval * 60 * 1000;
      autoRefreshRef.current = window.setInterval(() => {
        checkAllVersions().catch(console.error);
      }, interval);

      return () => {
        if (autoRefreshRef.current) {
          clearInterval(autoRefreshRef.current);
        }
      };
    }
  }, [settings.cache.autoRefreshEnabled, settings.cache.autoRefreshInterval, checkAllVersions]);

  const handleAddSoftware = useCallback(
    async (form: SoftwareFormData) => {
      // 检查重复
      const exists = softwares.some(
        (s) =>
          s.source.type === form.source.type &&
          s.source.identifier.toLowerCase() === form.source.identifier.toLowerCase()
      );
      if (exists) {
        toast.error("添加失败", { description: "该软件已存在，请勿重复添加" });
        throw new Error("软件已存在");
      }

      // 后端会先验证版本获取，成功才添加
      await addSoftware(form);
      toast.success("添加成功", { description: `已添加 ${form.name}` });
    },
    [addSoftware, softwares]
  );

  const handleUpdateSoftware = useCallback(
    async (id: string, form: SoftwareFormData) => {
      try {
        await updateSoftware(id, form);
        toast.success("更新成功", { description: `已更新 ${form.name}` });
      } catch (error) {
        toast.error("更新失败", { description: String(error) });
        throw error;
      }
    },
    [updateSoftware]
  );

  const handleDeleteSoftware = useCallback(
    async (id: string) => {
      const software = softwares.find((s) => s.id === id);
      if (!software) return;

      if (!window.confirm(`确定要删除 "${software.name}" 吗？`)) return;

      try {
        await deleteSoftware(id);
        toast.success("删除成功", { description: `已删除 ${software.name}` });
      } catch (error) {
        toast.error("删除失败", { description: String(error) });
      }
    },
    [softwares, deleteSoftware]
  );

  const handleRefresh = useCallback(
    async (id: string) => {
      setRefreshingId(id);
      try {
        await checkVersion(id, true);
        toast.success("刷新成功");
      } catch (error) {
        toast.error("刷新失败", { description: String(error) });
      } finally {
        setRefreshingId(null);
      }
    },
    [checkVersion]
  );

  const handleRefreshAll = useCallback(async () => {
    try {
      await checkAllVersions();
      toast.success("刷新成功", { description: "已更新所有软件版本信息" });
    } catch (error) {
      toast.error("刷新失败", { description: String(error) });
    }
  }, [checkAllVersions]);

  const handleEdit = useCallback((software: Software) => {
    setEditingSoftware(software);
    setEditDialogOpen(true);
  }, []);

  const handleSaveSettings = useCallback(
    async (newSettings: typeof settings) => {
      try {
        await saveSettings(newSettings);
        toast.success("设置已保存");
      } catch (error) {
        toast.error("保存失败", { description: String(error) });
        throw error;
      }
    },
    [saveSettings]
  );

  const handleClearCache = useCallback(async () => {
    try {
      await clearCache();
      toast.success("缓存已清除");
    } catch (error) {
      toast.error("清除失败", { description: String(error) });
      throw error;
    }
  }, [clearCache]);

  // Find the most recent check time
  const lastCheckedAt = softwares
    .filter((s) => s.lastCheckedAt)
    .sort((a, b) => {
      const dateA = a.lastCheckedAt ? new Date(a.lastCheckedAt).getTime() : 0;
      const dateB = b.lastCheckedAt ? new Date(b.lastCheckedAt).getTime() : 0;
      return dateB - dateA;
    })[0]?.lastCheckedAt;

  return (
    <div className="min-h-screen bg-background">
      <Header
        onAddClick={() => setAddDialogOpen(true)}
        onSettingsClick={() => setSettingsDialogOpen(true)}
        onRefreshClick={handleRefreshAll}
        isRefreshing={isChecking}
      />

      <main className="container mx-auto py-6 px-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <p className="text-muted-foreground">加载中...</p>
          </div>
        ) : (
          <SoftwareTable
            softwares={softwares}
            onRefresh={handleRefresh}
            onEdit={handleEdit}
            onDelete={handleDeleteSoftware}
            isRefreshing={refreshingId}
          />
        )}
      </main>

      <footer className="border-t py-3 px-4">
        <div className="container mx-auto flex items-center justify-between text-sm text-muted-foreground">
          <span>上次检查: {fromNow(lastCheckedAt)}</span>
          <span>共 {softwares.length} 个软件</span>
        </div>
      </footer>

      <AddSoftwareDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
        onSubmit={handleAddSoftware}
      />

      <EditSoftwareDialog
        open={editDialogOpen}
        onOpenChange={setEditDialogOpen}
        software={editingSoftware}
        onSubmit={handleUpdateSoftware}
      />

      <SettingsDialog
        open={settingsDialogOpen}
        onOpenChange={setSettingsDialogOpen}
        settings={settings}
        onSave={handleSaveSettings}
        onClearCache={handleClearCache}
      />

      <Toaster position="bottom-right" />
    </div>
  );
}

export default App;
