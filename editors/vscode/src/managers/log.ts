import type {
  Disposable,
  LogOutputChannel,
} from 'vscode';
import {
  window,
} from 'vscode';

/**
 * Singleton providing centralized logging to a VSCode output channel
 */
export class LogManager implements Disposable {
  private static instance: LogManager | undefined;
  private readonly channel: LogOutputChannel;

  private constructor () {
    this.channel = window.createOutputChannel('Typedown LSP', {
      log: true,
    });
  }

  static getInstance (): LogManager {
    if (!LogManager.instance) {
      LogManager.instance = new LogManager();
    }

    return LogManager.instance;
  }

  get mainChannel (): LogOutputChannel {
    return this.channel;
  }

  trace (message: string, ...args: unknown[]): void {
    this.channel.trace(message, ...args);
  }

  debug (message: string, ...args: unknown[]): void {
    this.channel.debug(message, ...args);
  }

  info (message: string, ...args: unknown[]): void {
    this.channel.info(message, ...args);
  }

  warn (message: string, ...args: unknown[]): void {
    this.channel.warn(message, ...args);
  }

  error (message: string | Error, ...args: unknown[]): void {
    this.channel.error(message, ...args);
  }

  dispose (): void {
    this.channel.dispose();
  }
}
