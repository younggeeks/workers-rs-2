import { describe, expect, test } from "vitest";
import { mf, mfUrl } from "./mf";
describe("vectorize", () => {
    test("describe returns index info", async () => {
        // 1. Dispatch a fetch to your worker that uses the binding
        const resp = await mf.dispatchFetch(`${mfUrl}vectorize/describe`);

        expect(resp.status).toBe(200);
        const body = await resp.text();
        const parsed = JSON.parse(body);
        // 2. Assert the response matches your detailed mock
        expect(parsed.name).toBe("VECTORIZE");
        expect(parsed.dimensions).toBe(2);
        expect(parsed.metric).toBe("cosine");
    });
});