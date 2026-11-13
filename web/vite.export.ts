import { resolve } from "node:path";
import type { Plugin } from "vite";

/** Names the file that ends the script element in a bundle, and the one that opens a comment in
 * it: neither may stand in the page as it is. */
const SCRIPT_END = /<\/(script)/gi;
const COMMENT_START = /<!--/g;

/** The modules of the app that the export swaps for its own: the call of the server and the
 * router. The screens import them as usual and do not know the difference. */
function swaps(src: string): Map<string, string> {
  return new Map([
    [resolve(src, "shared/api/commands.ts"), resolve(src, "export/commands.ts")],
    [resolve(src, "app/router.ts"), resolve(src, "export/router.ts")],
  ]);
}

function swapModules(src: string): Plugin {
  const table = swaps(src);
  return {
    name: "codeskimr-export-swap",
    enforce: "pre",
    async resolveId(source, importer, options) {
      if (!importer || !source.startsWith(".")) {
        return null;
      }
      const found = await this.resolve(source, importer, { ...options, skipSelf: true });
      return found ? (table.get(found.id) ?? null) : null;
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
