// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Fuzzy matching for the command palette (REQ-APP-004): case- and
 * diacritic-insensitive (fr/es/de/pt fold Latin accents; Arabic and
 * other scripts match as-is), substring beats subsequence, earlier
 * and tighter matches score higher.
 */

/** Lowercase + strip combining marks (Latin diacritics fold). */
export function normalize(text: string): string {
  return text
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase();
}

/**
 * Score a query against a target; null when it does not match.
 * Substring: 100 base, minus position and slack. Subsequence
 * (initials / scattered letters): 20 base, minus gaps.
 */
export function fuzzyScore(query: string, target: string): number | null {
  const q = normalize(query.trim());
  const t = normalize(target);
  if (q.length === 0) return 0;
  const substringAt = t.indexOf(q);
  if (substringAt >= 0) {
    return 100 - substringAt * 2 - (t.length - q.length);
  }
  // Subsequence walk.
  let at = 0;
  let gaps = 0;
  for (const char of q) {
    const found = t.indexOf(char, at);
    if (found < 0) return null;
    gaps += found - at;
    at = found + 1;
  }
  return 20 - gaps;
}

export interface FuzzyEntry<T> {
  /** The searchable text (label + id, pre-joined by the caller). */
  haystack: string;
  /** Stable tie-break order (KB order for command rows). */
  order: number;
  item: T;
}

/** Rank entries by score, then by their stable order. */
export function fuzzyRank<T>(
  query: string,
  entries: ReadonlyArray<FuzzyEntry<T>>,
): Array<FuzzyEntry<T> & { score: number }> {
  const scored: Array<FuzzyEntry<T> & { score: number }> = [];
  for (const entry of entries) {
    const score = fuzzyScore(query, entry.haystack);
    if (score !== null) scored.push({ ...entry, score });
  }
  scored.sort((a, b) => b.score - a.score || a.order - b.order);
  return scored;
}
