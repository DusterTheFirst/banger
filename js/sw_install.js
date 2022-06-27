// @ts-check

async function registerServiceWorker() {
    if ("serviceWorker" in navigator) {
        try {
            const registration = await navigator.serviceWorker.register(
                "/sw.js",
                {
                    scope: "/",
                    type: "classic",
                    updateViaCache: "all",
                }
            );

            if (registration.installing) {
                console.log("Service worker installing");
            } else if (registration.waiting) {
                console.log("Service worker installed");
            } else if (registration.active) {
                console.log("Service worker active");
            }
        } catch (error) {
            console.error(`Registration failed with ${error}`);
        }
    } else {
        console.warn("Service workers are not supported");
    }
}

registerServiceWorker();
