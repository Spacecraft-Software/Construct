# Spacecraft TypeScript Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for TypeScript 7.0+ (Go native compiler) systems programming. It provides complete, compile-checked configurations and skeletons for monorepo project references, Piscina worker threadpools, Zod boundary validation, and unit tests.

---

## 1. Project References & Incremental Configuration (TS 7.0)

To scale type-checking with the Go compiler, divide the project into references. The compiler compiles upstream projects and caches output declaration files.

### Root Configuration (`tsconfig.json`)
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "composite": true,
    "incremental": true,
    "declaration": true,
    "declarationMap": true,
    "noEmitOnError": true,
    "skipLibCheck": true
  },
  "files": [],
  "references": [
    { "path": "./packages/core" },
    { "path": "./packages/api" }
  ]
}
```

### Core Package Configuration (`packages/core/tsconfig.json`)
```json
{
  "extends": "../../tsconfig.json",
  "compilerOptions": {
    "outDir": "./dist",
    "tsBuildInfoFile": "./dist/.tsbuildinfo"
  },
  "include": ["src/**/*"]
}
```

### API Package Configuration (`packages/api/tsconfig.json`)
```json
{
  "extends": "../../tsconfig.json",
  "compilerOptions": {
    "outDir": "./dist",
    "tsBuildInfoFile": "./dist/.tsbuildinfo"
  },
  "include": ["src/**/*"],
  "references": [
    { "path": "../core" }
  ]
}
```

---

## 2. Piscina Worker Threads Skeleton (CPU Parallelism)

Do not block the single-threaded event loop with expensive calculations. Offload them to background worker threads using `piscina`.

### Parent Service (`src/telemetry-service.ts`)
```typescript
import { Piscina } from 'piscina';
import * as path from 'path';

export interface ComputeRequest {
  data: Float64Array;
}

export interface ComputeResponse {
  sum: number;
}

const workerPool = new Piscina({
  filename: path.resolve(__dirname, 'worker.js'),
  minThreads: 2,
  maxThreads: 8
});

export async function processTelemetryParallel(data: Float64Array): Promise<number> {
  if (data.length < 5000) {
    // Fall back to serial sum to avoid thread communication overhead
    return data.reduce((acc, val) => acc + val, 0);
  }
  
  const request: ComputeRequest = { data };
  const response: ComputeResponse = await workerPool.run(request);
  return response.sum;
}
```

### Worker Script (`src/worker.ts`)
```typescript
import { ComputeRequest, ComputeResponse } from './telemetry-service';

export default function (request: ComputeRequest): ComputeResponse {
  const { data } = request;
  let sum = 0;
  for (let i = 0; i < data.length; i++) {
    sum += data[i]!;
  }
  return { sum };
}
```

---

## 3. Zod Boundary Schema Validation

Validate all untrusted external data (network payloads, filesystem inputs) before casting it to internal TypeScript interfaces.

```typescript
import { z } from 'zod';

// Define schema (contains runtime check logic)
export const SensorReadingSchema = z.object({
  sensorId: z.string().min(1),
  temperature: z.number(),
  timestamp: z.string().datetime(), // ISO 8601 UTC
  status: z.union([z.literal('ok'), z.literal('error')])
});

// Infer TypeScript interface automatically
export type SensorReading = z.infer<typeof SensorReadingSchema>;

export function parseIncomingTelemetry(payload: string): Result<SensorReading, Error> {
  try {
    const rawData = JSON.parse(payload);
    // parse returns the validated object or throws an error
    const validated = SensorReadingSchema.parse(rawData);
    return { success: true, data: validated };
  } catch (err) {
    return { success: false, error: err as Error };
  }
}

type Result<T, E> = 
  | { success: true; data: T }
  | { success: false; error: E };
```

---

## 4. Discriminated Union Exhaustiveness (never assertion)

Ensure that all code paths are handled completely. If a new variant is added to a union, the compiler must fail to build.

```typescript
export type Command =
  | { type: 'start'; delay: number }
  | { type: 'stop' }
  | { type: 'restart'; clean: boolean };

export function assertNever(x: never): never {
  throw new Error(`Exhaustiveness check failed: unexpected object ${JSON.stringify(x)}`);
}

export function executeCommand(cmd: Command): void {
  switch (cmd.type) {
    case 'start':
      console.log(`Starting telemetry, delay: ${cmd.delay}`);
      break;
    case 'stop':
      console.log('Stopping telemetry');
      break;
    case 'restart':
      console.log(`Restarting telemetry, clean: ${cmd.clean}`);
      break;
    default:
      // If a new variant is added and not handled in the switch, 
      // the compiler will throw a type error because the type of 'cmd' is not 'never'
      assertNever(cmd);
  }
}
```

---

## 5. Testing: Vitest & fast-check

Test suites must check happy paths, errors, and invariants. Use `fast-check` to execute property-based tests.

```typescript
// test/telemetry.test.ts
import { describe, it, expect } from 'vitest';
import * as fc from 'fast-check';
import { executeCommand, Command } from '../src/commands';

describe('Telemetry Commands', () => {
  it('should parse and process commands cleanly', () => {
    const cmd: Command = { type: 'stop' };
    expect(() => executeCommand(cmd)).not.toThrow();
  });

  it('should satisfy list-reversing invariants', () => {
    fc.assert(
      fc.property(fc.array(fc.integer()), (arr) => {
        const rev = [...arr].reverse();
        const doubleRev = [...rev].reverse();
        expect(doubleRev).toEqual(arr);
      })
    );
  });
});
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **`any` / `as any` overrides** | Runtime type errors and crashes | Enable `strict: true` and replace `any` with `unknown` type guards. |
| **Dynamic property assignments** | V8 engine de-optimization (slow loop) | Initialize all properties inside the class constructor or object literal up front. |
| **Blocking main event loop** | Lagging request handlers or web socket drops | Offload heavy CPU-bound algorithms to a `Piscina` worker pool. |
| **Programmatic tsc API calls** | Webpack or ESLint errors on TS 7.0 | Install `@typescript/typescript6` and utilize its shims for programmatic tooling. |
| **Raw worker thread loops** | Out of memory / process limits | Use persistent worker threadpools (`Piscina`) instead of spawning raw workers on-demand. |
| **Silent parser errors** | Missing type fields crash downstream code | Parse network JSON inputs immediately using `Zod` schema models. |

---

## 7. Code Review Compliance Gate

Before merging TypeScript code, verify:
1. `strict: true` and strict type checks are enabled in `tsconfig.json`.
2. No `any` casting or `!` assertions are present in production code paths.
3. Every external data ingress point is bound by a Zod schema validation.
4. Hot objects maintain stable hidden classes (no dynamic property additions/deletions).
5. Monorepo dependencies are referenced cleanly with `composite: true`.
