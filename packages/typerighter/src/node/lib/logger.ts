import colors from 'picocolors';

export const logger = {
  info (message: string): void {
    console.log(colors.cyan(message));
  },

  warn (message: string): void {
    console.warn(colors.yellow(message));
  },

  error (message: string): void {
    console.error(colors.red(message));
  },
};
