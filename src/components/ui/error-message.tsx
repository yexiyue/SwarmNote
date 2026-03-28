import { cn } from "@/lib/utils";

interface ErrorMessageProps {
  error: string | null;
  className?: string;
}

export function ErrorMessage({ error, className }: ErrorMessageProps) {
  if (!error) return null;
  return <p className={cn("text-xs text-destructive", className)}>{error}</p>;
}
