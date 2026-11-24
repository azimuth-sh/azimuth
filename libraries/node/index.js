const VARIABLE = "AZIMUTH_CONTEXT";

// The report's own words for a source's position relative to Azimuth's applied value.
const BEFORE = "before azimuth";
const AZIMUTH = "azimuth";
const AFTER = "after azimuth";

const ENVIRONMENT = "environment";

// The source of a key mapped to a name. `process.env` has one value per name and no record of who
// wrote it, so the origin of the applied text is kept here and compared with the current text.
function environmentSource(name) {
  let applied = null;
  return {
    read() {
      const current = process.env[name];
      if (current === undefined) return { value: null, source: null };
      if (applied === null) return { value: current, source: { name: ENVIRONMENT, precedence: BEFORE } };
      if (current === applied) return { value: current, source: { name: AZIMUTH, precedence: AZIMUTH } };
      return { value: current, source: { name: ENVIRONMENT, precedence: AFTER } };
    },
    apply(text) {
      process.env[name] = text;
      applied = text;
    },
  };
}

function readLaunch() {
  const text = process.env[VARIABLE];
  if (text === undefined) return {};
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(`${VARIABLE} must be one JSON object of text values: ${error.message}`, { cause: error });
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`${VARIABLE} must be one JSON object of text values.`);
  }
  for (const [key, value] of Object.entries(parsed)) {
    if (typeof value !== "string") {
      throw new Error(`${VARIABLE} must be one JSON object of text values; "${key}" is not text.`);
    }
  }
  return parsed;
}

export function registerAzimuthContext(name, { applyingWhen, map }) {
  const applying = Boolean(applyingWhen());
  const launch = applying ? readLaunch() : {};

  const entries = Object.entries(map).map(([key, setting]) => [
    key,
    typeof setting === "string" ? environmentSource(setting) : setting,
  ]);

  for (const [key, source] of entries) {
    if (Object.hasOwn(launch, key)) source.apply(launch[key]);
  }

  return {
    report() {
      const context = {};
      for (const [key, source] of entries) {
        const { value, source: origin } = source.read();
        context[key] = {
          value: value ?? null,
          source: origin ? { name: origin.name, precedence: origin.precedence } : null,
          read: new Date().toISOString(),
        };
      }
      return JSON.stringify({ format: 1, component: name, applying: applying ? "possible" : "not possible", context });
    },
  };
}
