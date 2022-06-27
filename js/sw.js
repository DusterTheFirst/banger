// @ts-check
/// <reference lib="webworker" />

let CACHE_NAME = "sw-cache";
const STATIC_FILES = ["manifest.json"];

/**
 * Cache requests
 * @param {Request} request
 * @returns {Promise<Response>}
 */
async function cache_request(request) {
    const cache = await caches.open(CACHE_NAME);

    // Attempt to load from cache if offline
    if (!navigator.onLine) {
        const cached_response = await cache.match(request);
        if (cached_response !== undefined) {
            return cached_response;
        }
    }

    const response = await fetch(request).catch(() => cache.match(request));

    if (response === undefined) {
        return Response.error();
    }

    cache.put(request, response.clone());
    return response;
}

async function cache_static() {
    const cache = await caches.open(CACHE_NAME);
    await cache.addAll(STATIC_FILES);
}

(() => {
    // This is a little messy, but necessary to force type assertion
    // Same issue as in TS -> https://github.com/microsoft/TypeScript/issues/14877
    // prettier-ignore
    const self = /** @type {ServiceWorkerGlobalScope} */ (/** @type {unknown} */ (globalThis.self));

    self.addEventListener("fetch", (event) => {
        event.respondWith(cache_request(event.request));
    });

    self.addEventListener("install", (e) => {
        e.waitUntil(cache_static());
    });
})();
