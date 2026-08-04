import type {
  TdLogger,
} from './logger';
import {
  formatMs,
} from '@/shared';

export class ProgressLogger {
  private logger: TdLogger;
  private label: string;
  private start: number;

  constructor (logger: TdLogger, label: string) {
    this.logger = logger;
    this.label = label;
    this.start = performance.now();
    this.logger.start(label);
  }

  update (current: number, total: number) {
    if (!process.stdout.isTTY) return;

    const pct = Math.round((current / total) * 100);

    process.stdout.write(`\r  ${this.label} ${current}/${total} (${pct}%)`);
  }

  done (message: string) {
    if (process.stdout.isTTY) {
      process.stdout.write('\r\x1b[K');
    }

    const elapsed = Math.round(performance.now() - this.start);

    this.logger.success(`${message} (${formatMs(elapsed)})`);
  }
}
