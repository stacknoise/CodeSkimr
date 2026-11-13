// Starts the end-to-end server of the app (release build, in-memory keystore) for the acceptance
// scripts of WP-54 and hands out launch files. Needs `npm run build` in web/ first.
import { spawn } from "node:child_process";
import path from "node:path";
import { createInterface } from "node:readline";

export const root = path.resolve(import.meta.dirname, "../..");

/** Starts the server and resolves with its URL, a `launchFile()` function and `stop()`. */
export async function startServer() {
    const child = spawn(
        "cargo",
        [
            "test",
            "--release",
            "-p",
            "codeskimr",
            "e2e_server",
            "--",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ],
        {
            cwd: root,
            env: {
                ...process.env,
                CODESKIMR_E2E_UI_DIR: path.join(root, "web/dist"),
            },
        },
    );
    const waiting = [];
    const url = await new Promise((resolve) => {
        createInterface({ input: child.stdout }).on("line", (line) => {
            const found = /E2E_URL=(\S+)/.exec(line)?.[1];
            const file = /E2E_LAUNCH_FILE=(.+)$/.exec(line)?.[1];
            if (found) resolve(found);
            if (file) waiting.shift()?.(file);
        });
    });
    const launchFile = () =>
        new Promise((done) => {
            waiting.push(done);
            child.stdin.write("launch\n");
        });
    return { url, launchFile, stop: () => child.stdin.end() };
}
