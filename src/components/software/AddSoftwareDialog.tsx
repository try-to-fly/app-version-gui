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
  const [name, setName] = useState("");
  const [sourceType, setSourceType] = useState<SourceType>("github-release");
  const [identifier, setIdentifier] = useState("");
  const [localCommand, setLocalCommand] = useState("");
  const [versionArg, setVersionArg] = useState("--version");
  const [isSubmitting, setIsSubmitting] = useState(false);

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
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>添加软件</DialogTitle>
          <DialogDescription>
            添加要追踪版本的软件。支持 GitHub Release、Tags 和 Homebrew。
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
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
