import type { ClassValue } from "clsx"
import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Calcule un pourcentage entier (0 si total = 0) */
export function calcPercent(done: number, total: number): number {
  if (total === 0) return 0
  return Math.round((done / total) * 100)
}
