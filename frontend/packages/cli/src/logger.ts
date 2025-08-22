import { pino } from "pino";
import pretty from "pino-pretty";

const prettyStream = pretty({
  levelFirst: true,
  colorize: true,
});

const logger = pino<string>(
  {},
  process.env.NODE_ENV !== "production" ? prettyStream : undefined,
);

export default logger;
