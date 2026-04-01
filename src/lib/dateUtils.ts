export function formatRelativeTime(input: string | null): string {
  if (input == null) return "从未在线";

  const timestamp = new Date(input).getTime();
  const diff = Date.now() - timestamp;
  const minutes = Math.floor(diff / 60_000);

  if (minutes < 1) return "刚刚";
  if (minutes < 60) return `${minutes} 分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days} 天前`;

  return new Date(timestamp).toLocaleDateString();
}

export function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}
