import { Badge } from "@/components/ui/badge";
import type { UpdateStatus } from "@/types/software";

interface VersionBadgeProps {
  status: UpdateStatus;
}

export function VersionBadge({ status }: VersionBadgeProps) {
  switch (status) {
    case "up-to-date":
      return <Badge variant="success">最新</Badge>;
    case "update-available":
      return <Badge variant="warning">可更新</Badge>;
    case "unknown":
    default:
      return <Badge variant="secondary">未知</Badge>;
  }
}
