import { clearCache, layout, prepare, type PreparedText } from "@chenglou/pretext";

import type { Log } from "@/bindings";
import { LOG_ROW_MIN_HEIGHT, LOG_ROW_VERTICAL_PADDING } from "../constants";

export type LogTextMetrics = {
  contentWidth: number;
  font: string;
  letterSpacing: number;
  lineHeight: number;
};

type PreparedTextCacheEntry = {
  content: string;
  font: string;
  letterSpacing: number;
  prepared: PreparedText;
};

const preparedTextCache = new Map<number, PreparedTextCacheEntry>();

function canMeasureText() {
  const segmenter = (Intl as typeof Intl & { Segmenter?: unknown }).Segmenter;
  return typeof segmenter === "function" && typeof document !== "undefined";
}

function getPreparedText(log: Log, metrics: LogTextMetrics) {
  const cached = preparedTextCache.get(log.id);
  if (
    cached?.content === log.content &&
    cached.font === metrics.font &&
    cached.letterSpacing === metrics.letterSpacing
  ) {
    return cached.prepared;
  }

  const prepared = prepare(log.content, metrics.font, {
    letterSpacing: metrics.letterSpacing,
    whiteSpace: "pre-wrap",
    wordBreak: "normal",
  });
  preparedTextCache.set(log.id, {
    content: log.content,
    font: metrics.font,
    letterSpacing: metrics.letterSpacing,
    prepared,
  });
  return prepared;
}

export function estimateLogRowHeight(log: Log, metrics: LogTextMetrics | null) {
  if (!metrics || !canMeasureText() || metrics.contentWidth <= 0) return LOG_ROW_MIN_HEIGHT;

  const measurement = layout(
    getPreparedText(log, metrics),
    metrics.contentWidth,
    metrics.lineHeight,
  );
  return Math.max(metrics.lineHeight, measurement.height) + LOG_ROW_VERTICAL_PADDING;
}

export function readLogTextMetrics(element: HTMLElement): LogTextMetrics {
  const styles = window.getComputedStyle(element);
  const letterSpacing = Number.parseFloat(styles.letterSpacing);
  const lineHeight = Number.parseFloat(styles.lineHeight);

  return {
    contentWidth: element.getBoundingClientRect().width,
    font: `${styles.fontStyle} ${styles.fontWeight} ${styles.fontSize} ${styles.fontFamily}`,
    letterSpacing: Number.isFinite(letterSpacing) ? letterSpacing : 0,
    lineHeight: Number.isFinite(lineHeight) ? lineHeight : LOG_ROW_MIN_HEIGHT,
  };
}

export function clearLogRowMeasurementCache() {
  preparedTextCache.clear();
  clearCache();
}
