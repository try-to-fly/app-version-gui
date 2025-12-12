import { useState } from "react";
import { RefreshCw, MoreHorizontal, Pencil, Trash2 } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { VersionBadge } from "./VersionBadge";
import { fromNow } from "@/lib/time";
import type { Software } from "@/types/software";
import { getUpdateStatus } from "@/types/software";

interface SoftwareTableProps {
  softwares: Software[];
  onRefresh: (id: string) => void;
  onEdit: (software: Software) => void;
  onDelete: (id: string) => void;
  isRefreshing: string | null;
}

export function SoftwareTable({
  softwares,
  onRefresh,
  onEdit,
  onDelete,
  isRefreshing,
}: SoftwareTableProps) {
  const [deleteTarget, setDeleteTarget] = useState<Software | null>(null);

  const handleDeleteClick = (software: Software) => {
    setDeleteTarget(software);
  };

  const handleConfirmDelete = () => {
    if (deleteTarget) {
      onDelete(deleteTarget.id);
      setDeleteTarget(null);
    }
  };

  if (softwares.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
        <p className="text-lg">暂无软件</p>
        <p className="text-sm">点击右上角"添加"按钮添加要追踪的软件</p>
      </div>
    );
  }

  return (
    <>
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[200px]">名称</TableHead>
          <TableHead>最新版本</TableHead>
          <TableHead>更新时间</TableHead>
          <TableHead>本地版本</TableHead>
          <TableHead>状态</TableHead>
          <TableHead className="w-[100px]">操作</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {[...softwares]
          .sort((a, b) => {
            const dateA = a.publishedAt ? new Date(a.publishedAt).getTime() : 0;
            const dateB = b.publishedAt ? new Date(b.publishedAt).getTime() : 0;
            return dateB - dateA;
          })
          .map((software) => {
          const status = getUpdateStatus(
            software.latestVersion,
            software.localVersion
          );
          return (
            <TableRow key={software.id}>
              <TableCell className="font-medium">{software.name}</TableCell>
              <TableCell>{software.latestVersion || "-"}</TableCell>
              <TableCell className="text-muted-foreground">
                {fromNow(software.publishedAt)}
              </TableCell>
              <TableCell>{software.localVersion || "-"}</TableCell>
              <TableCell>
                <VersionBadge status={status} />
              </TableCell>
              <TableCell>
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => onRefresh(software.id)}
                    disabled={isRefreshing === software.id}
                  >
                    <RefreshCw
                      className={`h-4 w-4 ${
                        isRefreshing === software.id ? "animate-spin" : ""
                      }`}
                    />
                  </Button>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="icon">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => onEdit(software)}>
                        <Pencil className="h-4 w-4 mr-2" />
                        编辑
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        className="text-destructive"
                        onClick={() => handleDeleteClick(software)}
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        删除
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </TableCell>
            </TableRow>
          );
        })}
      </TableBody>
    </Table>

    <Dialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>
            确定要删除「{deleteTarget?.name}」吗？此操作无法撤销。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={() => setDeleteTarget(null)}>
            取消
          </Button>
          <Button variant="destructive" onClick={handleConfirmDelete}>
            删除
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
    </>
  );
}
