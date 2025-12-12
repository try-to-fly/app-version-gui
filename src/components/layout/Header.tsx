import { Package, Settings, RefreshCw, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

interface HeaderProps {
  onAddClick: () => void;
  onSettingsClick: () => void;
  onRefreshClick: () => void;
  isRefreshing: boolean;
}

export function Header({
  onAddClick,
  onSettingsClick,
  onRefreshClick,
  isRefreshing,
}: HeaderProps) {
  return (
    <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="flex h-14 items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <Package className="h-6 w-6" />
          <h1 className="text-lg font-semibold">Version Tracker</h1>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={onRefreshClick}
            disabled={isRefreshing}
          >
            <RefreshCw
              className={`h-4 w-4 mr-1 ${isRefreshing ? "animate-spin" : ""}`}
            />
            刷新全部
          </Button>
          <Button variant="outline" size="icon" onClick={onSettingsClick}>
            <Settings className="h-4 w-4" />
          </Button>
          <Button size="sm" onClick={onAddClick}>
            <Plus className="h-4 w-4 mr-1" />
            添加
          </Button>
        </div>
      </div>
    </header>
  );
}
