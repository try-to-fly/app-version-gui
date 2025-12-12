import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import "dayjs/locale/zh-cn";

dayjs.extend(relativeTime);
dayjs.locale("zh-cn");

export function fromNow(date: string | null | undefined): string {
  if (!date) return "从未检查";
  return dayjs(date).fromNow();
}

export function formatDate(date: string | null | undefined): string {
  if (!date) return "-";
  return dayjs(date).format("YYYY-MM-DD HH:mm");
}
