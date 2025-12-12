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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Software, SoftwareFormData, SourceType } from "@/types/software";

interface EditSoftwareDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  software: Software | null;
  onSubmit: (id: string, form: SoftwareFormData) => Promise<void>;
}

export function EditSoftwareDialog({
  open,
  onOpenChange,
  software,
  onSubmit,
}: EditSoftwareDialogProps) {
  const [name, setName] = useState("");
  const [sourceType, setSourceType] = useState<SourceType>("github-release");
  const [identifier, setIdentifier] = useState("");
  const [localCommand, setLocalCommand] = useState("");
  const [versionArg, setVersionArg] = useState("--version");
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (software) {
      setName(software.name);
      setSourceType(software.source.type);
      setIdentifier(software.source.identifier);
      setLocalCommand(software.localVersionConfig?.command || "");
      setVersionArg(software.localVersionConfig?.versionArg || "--version");
    }
  }, [software]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!software || !name || !identifier) return;

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

      await onSubmit(software.id, form);
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to update software:", error);
    } finally {
      setIsSubmitting(false);
    }
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
          <DialogTitle>编辑软件</DialogTitle>
          <DialogDescription>修改软件的追踪配置。</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="edit-name">名称</Label>
            <Input
              id="edit-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="软件显示名称"
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="edit-sourceType">数据源类型</Label>
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
            <Label htmlFor="edit-identifier">标识符</Label>
            <Input
              id="edit-identifier"
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
              <Label htmlFor="edit-localCommand">本地命令</Label>
              <Input
                id="edit-localCommand"
                value={localCommand}
                onChange={(e) => setLocalCommand(e.target.value)}
                placeholder="如 git, node, rustc"
              />
            </div>
            {localCommand && (
              <div className="space-y-2 mt-2">
                <Label htmlFor="edit-versionArg">版本参数</Label>
                <Input
                  id="edit-versionArg"
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
              {isSubmitting ? "保存中..." : "保存"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
