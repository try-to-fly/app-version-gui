import { useState } from "react";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { SoftwareFormData, SourceType } from "@/types/software";

interface AddSoftwareDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (form: SoftwareFormData) => Promise<void>;
}

export function AddSoftwareDialog({
  open,
  onOpenChange,
  onSubmit,
}: AddSoftwareDialogProps) {
  const [url, setUrl] = useState("");
  const [name, setName] = useState("");
  const [sourceType, setSourceType] = useState<SourceType>("github-release");
  const [identifier, setIdentifier] = useState("");
  const [localCommand, setLocalCommand] = useState("");
  const [versionArg, setVersionArg] = useState("--version");
  const [isSubmitting, setIsSubmitting] = useState(false);

  // 解析 GitHub URL
  const parseGitHubUrl = (inputUrl: string) => {
    const trimmedUrl = inputUrl.trim();

    // 匹配 GitHub URL 格式: https://github.com/owner/repo[/releases|/tags|...]
    const githubUrlPattern = /^https?:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\/(?:releases|tags|issues|pulls|actions|tree|blob|commit|compare|wiki|pulse|graphs|network|settings).*)?(?:\.git)?$/i;
    const match = trimmedUrl.match(githubUrlPattern);

    if (match) {
      const owner = match[1];
      const repo = match[2].replace(/\.git$/, '');
      const repoIdentifier = `${owner}/${repo}`;

      // 根据 URL 路径判断数据源类型
      if (trimmedUrl.includes('/tags')) {
        setSourceType('github-tags');
      } else {
        setSourceType('github-release');
      }

      // 设置标识符
      setIdentifier(repoIdentifier);

      // 设置默认名称（如果名称为空）
      if (!name) {
        setName(repo);
      }

      return true;
    }

    return false;
  };

  // 处理 URL 输入变化
  const handleUrlChange = (value: string) => {
    setUrl(value);
    parseGitHubUrl(value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !identifier) return;

    setIsSubmitting(true);
    try {
      const form: SoftwareFormData = {
        name,
        source: {
          type: sourceType,
          identifier,
        },
        localVersionConfig: localCommand
          ? {
              command: localCommand,
              versionArg: versionArg || undefined,
            }
          : undefined,
      };

      await onSubmit(form);
      resetForm();
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to add software:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const resetForm = () => {
    setUrl("");
    setName("");
    setSourceType("github-release");
    setIdentifier("");
    setLocalCommand("");
    setVersionArg("--version");
  };

  const getIdentifierPlaceholder = () => {
    switch (sourceType) {
      case "github-release":
      case "github-tags":
        return "owner/repo (如 facebook/react)";
      case "homebrew":
        return "formula 名称 (如 git)";
      case "npm":
        return "包名 (如 react, @types/node)";
      case "pypi":
        return "包名 (如 django, requests)";
      case "cargo":
        return "crate 名称 (如 tokio, serde)";
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>添加软件</DialogTitle>
          <DialogDescription>
            添加要追踪版本的软件。支持 GitHub、Homebrew、npm、PyPI 和 Cargo。
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="url">GitHub URL（可选）</Label>
            <Input
              id="url"
              value={url}
              onChange={(e) => handleUrlChange(e.target.value)}
              placeholder="粘贴 GitHub 链接自动填充，如 https://github.com/owner/repo"
            />
            <p className="text-xs text-muted-foreground">
              输入 GitHub 链接可自动解析名称和标识符
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="name">名称</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="软件显示名称"
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="sourceType">数据源类型</Label>
            <Select
              value={sourceType}
              onValueChange={(v) => setSourceType(v as SourceType)}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="github-release">GitHub Release</SelectItem>
                <SelectItem value="github-tags">GitHub Tags</SelectItem>
                <SelectItem value="homebrew">Homebrew</SelectItem>
                <SelectItem value="npm">npm Registry</SelectItem>
                <SelectItem value="pypi">PyPI</SelectItem>
                <SelectItem value="cargo">crates.io (Cargo)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="identifier">标识符</Label>
            <Input
              id="identifier"
              value={identifier}
              onChange={(e) => setIdentifier(e.target.value)}
              placeholder={getIdentifierPlaceholder()}
              required
            />
          </div>

          <div className="border-t pt-4">
            <p className="text-sm text-muted-foreground mb-3">
              本地版本检测（可选）
            </p>
            <div className="space-y-2">
              <Label htmlFor="localCommand">本地命令</Label>
              <Input
                id="localCommand"
                value={localCommand}
                onChange={(e) => setLocalCommand(e.target.value)}
                placeholder="如 git, node, rustc"
              />
            </div>
            {localCommand && (
              <div className="space-y-2 mt-2">
                <Label htmlFor="versionArg">版本参数</Label>
                <Input
                  id="versionArg"
                  value={versionArg}
                  onChange={(e) => setVersionArg(e.target.value)}
                  placeholder="--version"
                />
              </div>
            )}
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              取消
            </Button>
            <Button type="submit" disabled={isSubmitting || !name || !identifier}>
              {isSubmitting ? "添加中..." : "添加"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
