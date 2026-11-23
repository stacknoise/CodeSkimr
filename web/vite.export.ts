import { resolve } from "node:path";
import type { Plugin } from "vite";

const BACKSLASH = /\\/g;

const DRIVE_LETTER = /^[A-Za-z]:/;

/** The path with forward slashes and a small drive letter, the way Vite names modules (Windows
 * writes the letter in either case). */
function normalizePath(path: string): string {
  return path.replace(BACKSLASH, "/").replace(DRIVE_LETTER, (drive) => drive.toLowerCase());
}

/** Names the file that ends the script element in a bundle, and the one that opens a comment in
 * it: neither may stand in the page as it is. */
const SCRIPT_END = /<\/(script)/gi;
const COMMENT_START = /<!--/g;

/** The modules of the app that the export swaps for its own: the call of the server and the
 * router. The screens import them as usual and do not know the difference. The keys use forward
 * slashes and a small drive letter, because the ids of Vite do, on Windows too (a path from
 * `resolve` has backslashes there, and the swap silently did not happen: the page then called the
 * server). The values are the own modules as paths for `resolve` of the build. */
export function swaps(src: string): Map<string, string> {
  return new Map([
    [normalizePath(resolve(src, "shared/api/commands.ts")), resolve(src, "export/commands.ts")],
    [normalizePath(resolve(src, "app/router.ts")), resolve(src, "export/router.ts")],
  ]);
}

/** Looks up the swap of a resolved module id. */
export function swapFor(table: Map<string, string>, id: string): string | null {
  return table.get(normalizePath(id)) ?? null;
}

/** Swaps the modules of `swaps`. The id that it returns is the one the build itself gives the
 * module of the swap when another file imports it (`./commands` in `export/main.tsx`): one module
 * under two ids is two copies, and the data that `installData` stores would not reach the
 * screens (the explorer then said that it could not load the structure). */
export function swapModules(src: string): Plugin {
  const table = swaps(src);
  const done = new Set<string>();
  return {
    name: "codeskimr-export-swap",
    enforce: "pre",
    async resolveId(source, importer, options) {
      if (!importer || !source.startsWith(".")) {
        return null;
      }
      const found = await this.resolve(source, importer, { ...options, skipSelf: true });
      const target = found ? swapFor(table, found.id) : null;
      if (target === null) {
        return null;
      }
      done.add(target);
      const own = await this.resolve(target, importer, { ...options, skipSelf: true });
      return own?.id ?? target;
    },
    buildEnd(error) {
      if (!error && done.size !== table.size) {
        this.error("the export build did not swap the call of the server and the router");
      }
    },
  };
}

/** Text that can stay inside a script element: `</script` and `<!--` are written so the browser
 * does not read them as markup; as part of a string or a regular expression they mean the same. */
export function safeInScript(code: string): string {
  return code.replace(SCRIPT_END, "<\\/$1").replace(COMMENT_START, "<\\!--");
}

/** Puts the script and the styles of the build into the page, so it is one file. The page keeps
 * its place for the data; `codeskimr export` fills it. */
function inlineIntoPage(): Plugin {
  return {
    name: "codeskimr-export-inline",
    enforce: "post",
    generateBundle(_options, bundle) {
      const page = Object.values(bundle).find(
        (item) => item.type === "asset" && item.fileName.endsWith(".html"),
      );
      if (page?.type !== "asset") {
        return;
      }
      let html = String(page.source);
      for (const [name, item] of Object.entries(bundle)) {
        const file = name.split("/").pop() ?? name;
        if (item.type === "chunk") {
          const tag = new RegExp(`<script[^>]*src="[^"]*${escape(file)}"[^>]*></script>`);
          html = html.replace(
            tag,
            () => `<script type="module">${safeInScript(item.code)}</script>`,
          );
          delete bundle[name];
        } else if (name.endsWith(".css")) {
          const tag = new RegExp(`<link[^>]*href="[^"]*${escape(file)}"[^>]*>`);
          html = html.replace(tag, () => `<style>${safeInScript(String(item.source))}</style>`);
          delete bundle[name];
        }
      }
      page.source = html;
    },
  };
}

function escape(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** The plugins of the export build. */
export function exportPlugins(src: string): Plugin[] {
  return [swapModules(src), inlineIntoPage()];
}
