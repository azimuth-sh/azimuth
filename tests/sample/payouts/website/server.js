import http from "node:http";
import { registerAzimuthContext } from "@azimuth-sh/context";

// `website` of the invented payouts product, written the way the design's example shows. The tests
// drive it with FIXTURE_* variables and read it through the endpoints under /fixture, which belong
// to this fixture and not to the library.

const env = (name) => process.env[name];
const mode = env("FIXTURE_MODE");

const state = {
  stack: env(env("FIXTURE_CONDITION_VARIABLE") ?? "PAYOUTS_TEST_STACK") === "1",
  conditionCalls: 0,
  seenAtCondition: null,
};

const instantName = env("FIXTURE_INSTANT_NAME") ?? "FEATURE_INSTANT_PAYOUTS";
const component = env("FIXTURE_COMPONENT") ?? "website";

const region = {
  value: env("FIXTURE_REGION_VALUE") ?? null,
  lastApplied: null,
  reads: 0,
  applies: 0,
  read() {
    this.reads += 1;
    const source =
      env("FIXTURE_REGION_SOURCE") === "none"
        ? null
        : {
            name: env("FIXTURE_REGION_NAME") ?? "region-flags",
            precedence:
              env("FIXTURE_REGION_PRECEDENCE") ??
              (this.lastApplied === null ? "before azimuth" : "azimuth"),
          };
    return { value: this.value, source };
  },
  apply(text) {
    this.applies += 1;
    this.lastApplied = text;
    this.value = text;
  },
};

const sideStore = { value: env("FIXTURE_SIDE") ?? null };
const sideTeam = {
  lastApplied: null,
  read() {
    return {
      value: sideStore.value,
      source: sideStore.value === null ? null : { name: "side-store", precedence: "before azimuth" },
    };
  },
  apply(text) {
    this.lastApplied = text;
    sideStore.value = text;
  },
};

function condition() {
  state.conditionCalls += 1;
  state.seenAtCondition = process.env[instantName] ?? null;
  return state.stack;
}

const maps = {
  instant: { "payouts.instant": instantName },
  region: { "payouts.region": region },
  wide: {
    "payouts.instant": instantName,
    "payouts.second": "FEATURE_SECOND_PAYOUTS",
    "payouts.region": region,
    "payouts.side": "FEATURE_SIDE_PAYOUTS",
    "payouts.sideteam": sideTeam,
  },
  default: { "payouts.instant": instantName, "payouts.region": region },
};

let context;
try {
  context = registerAzimuthContext(component, {
    applyingWhen: condition,
    map: maps[env("FIXTURE_KEYS") ?? "default"],
  });
} catch (error) {
  process.stderr.write(`registration failed: ${error.message}\n`, () => process.exit(1));
  await new Promise(() => {});
}

for (const pair of (env("FIXTURE_LATER") ?? "").split(";").filter(Boolean)) {
  const at = pair.indexOf("=");
  process.env[pair.slice(0, at)] = pair.slice(at + 1);
}

if (mode === "inspect") {
  const library = await import("@azimuth-sh/context");
  console.log(
    JSON.stringify({
      exports: Object.keys(library),
      contextProperties: Object.getOwnPropertyNames(context).concat(
        Object.getOwnPropertyNames(Object.getPrototypeOf(context)),
      ),
      reportType: typeof context.report(),
    }),
  );
} else if (mode === "console") {
  console.log(context.report());
  process.stdin.resume();
  process.stdin.on("end", () => process.exit(0));
} else {
  const json = (response, value) => {
    response.setHeader("content-type", "application/json");
    response.end(typeof value === "string" ? value : JSON.stringify(value));
  };
  const routes = {
    "/fixture/report": (query, response) => json(response, context.report()),
    "/fixture/envvar": (query, response) => json(response, { value: process.env[query.get("name")] ?? null }),
    "/fixture/env": (query, response) => {
      if (query.has("value")) process.env[query.get("name")] = query.get("value");
      else delete process.env[query.get("name")];
      json(response, {});
    },
    "/fixture/condition": (query, response) => {
      state.stack = query.get("value") === "1";
      json(response, {});
    },
    "/fixture/region": (query, response) => {
      if (query.has("value")) region.value = query.get("value");
      json(response, { reads: region.reads, applies: region.applies, lastApplied: region.lastApplied });
    },
    "/fixture/state": (query, response) =>
      json(response, {
        conditionCalls: state.conditionCalls,
        seenAtCondition: state.seenAtCondition,
        sideTeamApplied: sideTeam.lastApplied,
      }),
  };
  if (env("FIXTURE_NO_REPORT_PATH") !== "1") {
    routes["/internal/azimuth"] = (query, response) => json(response, context.report());
  }
  const server = http.createServer((request, response) => {
    const url = new URL(request.url, "http://127.0.0.1");
    const route = routes[url.pathname];
    if (!route) {
      response.statusCode = 404;
      response.end();
      return;
    }
    route(url.searchParams, response);
  });
  server.listen(0, "127.0.0.1", () => {
    console.log(`listening http://127.0.0.1:${server.address().port}`);
  });
}
