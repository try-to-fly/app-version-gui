// 数据源类型
export type SourceType = "github-release" | "github-tags" | "homebrew";

// 数据源配置
export interface SourceConfig {
  type: SourceType;
  // GitHub: "owner/repo", Homebrew: "formula-name"
  identifier: string;
}

// 本地版本检测配置
export interface LocalVersionConfig {
  // 本地命令名称，如 "git", "node"
  command: string;
  // 版本参数，默认 "--version"
  versionArg?: string;
}

// 软件条目
export interface Software {
  id: string;
  name: string;
  source: SourceConfig;
  // 可选的本地版本检测配置
  localVersionConfig?: LocalVersionConfig;
  latestVersion: string | null;
  localVersion: string | null;
  // 远程版本发布时间 (ISO 8601)
  publishedAt: string | null;
  // 最后检查时间 (ISO 8601)
  lastCheckedAt: string | null;
  enabled: boolean;
}

// 新建/编辑软件表单
export interface SoftwareFormData {
  name: string;
  source: SourceConfig;
  localVersionConfig?: LocalVersionConfig;
}

// 版本检查结果
export interface VersionCheckResult {
  softwareId: string;
  latestVersion: string;
  localVersion: string | null;
  publishedAt: string | null;
  hasUpdate: boolean;
}

// 缓存配置
export interface CacheConfig {
  // 缓存有效期（分钟）
  ttlMinutes: number;
  // 是否启用自动刷新
  autoRefreshEnabled: boolean;
  // 自动刷新间隔（分钟）
  autoRefreshInterval: number;
}

// 应用设置
export interface AppSettings {
  cache: CacheConfig;
  // 可选的 GitHub Token（提高 API 限额）
  githubToken?: string;
}

// 更新状态
export type UpdateStatus = "up-to-date" | "update-available" | "unknown";

// 获取更新状态
export function getUpdateStatus(
  latestVersion: string | null,
  localVersion: string | null
): UpdateStatus {
  if (!latestVersion || !localVersion) {
    return "unknown";
  }
  // 简单的版本比较，移除 'v' 前缀后比较
  const latest = latestVersion.replace(/^v/, "");
  const local = localVersion.replace(/^v/, "");
  if (latest === local) {
    return "up-to-date";
  }
  return "update-available";
}

// 数据源类型显示名称
export const SOURCE_TYPE_LABELS: Record<SourceType, string> = {
  "github-release": "GitHub Release",
  "github-tags": "GitHub Tags",
  homebrew: "Homebrew",
};
