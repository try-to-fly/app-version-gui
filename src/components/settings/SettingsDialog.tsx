import { useState, useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type { AppSettings, ThemeMode, NotificationConfig } from "@/types/software";
import { THEME_MODE_LABELS, DEFAULT_NOTIFICATION_CONFIG } from "@/types/software";
import { Sun, Moon, Monitor, Bell, AlertTriangle } from "lucide-react";

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  settings: AppSettings;
  onSave: (settings: AppSettings) => Promise<void>;
  onClearCache: () => Promise<void>;
}

const THEME_MODES: { mode: ThemeMode; icon: typeof Sun; label: string }[] = [
  { mode: "light", icon: Sun, label: THEME_MODE_LABELS.light },
  { mode: "dark", icon: Moon, label: THEME_MODE_LABELS.dark },
  { mode: "system", icon: Monitor, label: THEME_MODE_LABELS.system },
];

export function SettingsDialog({
  open,
  onOpenChange,
  settings,
  onSave,
  onClearCache,
}: SettingsDialogProps) {
  const [ttlMinutes, setTtlMinutes] = useState(30);
  const [autoRefreshEnabled, setAutoRefreshEnabled] = useState(true);
  const [autoRefreshInterval, setAutoRefreshInterval] = useState(60);
  const [githubToken, setGithubToken] = useState("");
  const [theme, setTheme] = useState<ThemeMode>("system");
  const [notification, setNotification] = useState<NotificationConfig>(DEFAULT_NOTIFICATION_CONFIG);
  const [isSaving, setIsSaving] = useState(false);
  const [isClearingCache, setIsClearingCache] = useState(false);

  useEffect(() => {
    if (settings) {
      setTtlMinutes(settings.cache.ttlMinutes);
      setAutoRefreshEnabled(settings.cache.autoRefreshEnabled);
      setAutoRefreshInterval(settings.cache.autoRefreshInterval);
      setGithubToken(settings.githubToken || "");
      setTheme(settings.theme || "system");
      setNotification(settings.notification || DEFAULT_NOTIFICATION_CONFIG);
    }
  }, [settings]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave({
        cache: {
          ttlMinutes,
          autoRefreshEnabled,
          autoRefreshInterval,
        },
        githubToken: githubToken || undefined,
        theme,
        notification,
      });
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to save settings:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleClearCache = async () => {
    setIsClearingCache(true);
    try {
      await onClearCache();
    } catch (error) {
      console.error("Failed to clear cache:", error);
    } finally {
      setIsClearingCache(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px] max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>设置</DialogTitle>
          <DialogDescription>配置应用程序的缓存和 API 设置。</DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto space-y-6 py-4 pr-2">
          <div className="space-y-4">
            <h3 className="text-sm font-medium">主题</h3>
            <div className="flex gap-2">
              {THEME_MODES.map(({ mode, icon: Icon, label }) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => setTheme(mode)}
                  className={`flex-1 flex flex-col items-center gap-2 p-3 rounded-lg border transition-all ${
                    theme === mode
                      ? "border-primary bg-primary/5"
                      : "border-border hover:border-primary/50"
                  }`}
                >
                  <Icon className="w-5 h-5" />
                  <span className="text-xs">{label}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-4 border-t pt-4">
            <h3 className="text-sm font-medium">缓存设置</h3>

            <div className="space-y-2">
              <Label htmlFor="ttl">缓存有效期（分钟）</Label>
              <Input
                id="ttl"
                type="number"
                min={1}
                max={1440}
                value={ttlMinutes}
                onChange={(e) => setTtlMinutes(Number(e.target.value))}
              />
              <p className="text-xs text-muted-foreground">
                版本信息在此时间内将使用缓存数据
              </p>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="autoRefresh">自动刷新</Label>
                <p className="text-xs text-muted-foreground">
                  定时自动检查版本更新
                </p>
              </div>
              <Switch
                id="autoRefresh"
                checked={autoRefreshEnabled}
                onCheckedChange={setAutoRefreshEnabled}
              />
            </div>

            {autoRefreshEnabled && (
              <div className="space-y-2">
                <Label htmlFor="interval">刷新间隔（分钟）</Label>
                <Input
                  id="interval"
                  type="number"
                  min={5}
                  max={1440}
                  value={autoRefreshInterval}
                  onChange={(e) => setAutoRefreshInterval(Number(e.target.value))}
                />
              </div>
            )}

            <Button
              variant="outline"
              size="sm"
              onClick={handleClearCache}
              disabled={isClearingCache}
            >
              {isClearingCache ? "清除中..." : "清除缓存"}
            </Button>
          </div>

          <div className="space-y-4 border-t pt-4">
            <h3 className="text-sm font-medium">GitHub API</h3>

            <div className="space-y-2">
              <Label htmlFor="token">GitHub Token（可选）</Label>
              <Input
                id="token"
                type="password"
                value={githubToken}
                onChange={(e) => setGithubToken(e.target.value)}
                placeholder="ghp_xxxx..."
              />
              <p className="text-xs text-muted-foreground">
                配置 Token 可将 API 限额从 60 次/小时提升至 5000 次/小时
              </p>
            </div>
          </div>

          <div className="space-y-4 border-t pt-4">
            <h3 className="text-sm font-medium flex items-center gap-2">
              <Bell className="w-4 h-4" />
              通知设置
            </h3>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="notificationEnabled">启用通知</Label>
                <p className="text-xs text-muted-foreground">
                  检测到新版本时发送系统通知
                </p>
              </div>
              <Switch
                id="notificationEnabled"
                checked={notification.enabled}
                onCheckedChange={(checked) =>
                  setNotification((prev) => ({ ...prev, enabled: checked }))
                }
              />
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="testMode" className="flex items-center gap-1">
                  测试模式
                  <AlertTriangle className="w-3 h-3 text-yellow-500" />
                </Label>
                <p className="text-xs text-muted-foreground">
                  即使没有更新也发送通知（用于测试）
                </p>
              </div>
              <Switch
                id="testMode"
                checked={notification.testMode}
                onCheckedChange={(checked) =>
                  setNotification((prev) => ({ ...prev, testMode: checked }))
                }
              />
            </div>

            {notification.testMode && (
              <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md p-3">
                <p className="text-xs text-yellow-700 dark:text-yellow-300 flex items-center gap-1">
                  <AlertTriangle className="w-3 h-3" />
                  测试模式已开启，每次后台检查都会发送通知，请在测试完成后关闭
                </p>
              </div>
            )}

            {notification.enabled && (
              <>
                <div className="space-y-3 pl-2 border-l-2 border-muted">
                  <p className="text-xs text-muted-foreground">通知版本类型：</p>
                  <div className="flex items-center justify-between">
                    <Label htmlFor="notifyMajor" className="text-sm font-normal">
                      主版本更新 (如 1.0 → 2.0)
                    </Label>
                    <Switch
                      id="notifyMajor"
                      checked={notification.notifyOnMajor}
                      onCheckedChange={(checked) =>
                        setNotification((prev) => ({ ...prev, notifyOnMajor: checked }))
                      }
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <Label htmlFor="notifyMinor" className="text-sm font-normal">
                      次版本更新 (如 1.0 → 1.1)
                    </Label>
                    <Switch
                      id="notifyMinor"
                      checked={notification.notifyOnMinor}
                      onCheckedChange={(checked) =>
                        setNotification((prev) => ({ ...prev, notifyOnMinor: checked }))
                      }
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <Label htmlFor="notifyPatch" className="text-sm font-normal">
                      补丁版本更新 (如 1.0.0 → 1.0.1)
                    </Label>
                    <Switch
                      id="notifyPatch"
                      checked={notification.notifyOnPatch}
                      onCheckedChange={(checked) =>
                        setNotification((prev) => ({ ...prev, notifyOnPatch: checked }))
                      }
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <Label htmlFor="notifyPrerelease" className="text-sm font-normal">
                      预发布版本 (alpha, beta, rc)
                    </Label>
                    <Switch
                      id="notifyPrerelease"
                      checked={notification.notifyOnPrerelease}
                      onCheckedChange={(checked) =>
                        setNotification((prev) => ({ ...prev, notifyOnPrerelease: checked }))
                      }
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label>静默时段</Label>
                  <div className="flex items-center gap-2">
                    <Input
                      type="number"
                      min={0}
                      max={23}
                      value={notification.silentStartHour ?? ""}
                      onChange={(e) =>
                        setNotification((prev) => ({
                          ...prev,
                          silentStartHour: e.target.value ? Number(e.target.value) : null,
                        }))
                      }
                      placeholder="22"
                      className="w-20"
                    />
                    <span className="text-sm text-muted-foreground">点 至</span>
                    <Input
                      type="number"
                      min={0}
                      max={23}
                      value={notification.silentEndHour ?? ""}
                      onChange={(e) =>
                        setNotification((prev) => ({
                          ...prev,
                          silentEndHour: e.target.value ? Number(e.target.value) : null,
                        }))
                      }
                      placeholder="8"
                      className="w-20"
                    />
                    <span className="text-sm text-muted-foreground">点</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    在此时段内不发送通知（留空则不限制）
                  </p>
                </div>
              </>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
            {isSaving ? "保存中..." : "保存"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
