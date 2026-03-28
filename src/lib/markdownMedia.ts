import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Pre-process markdown on load: convert relative media paths to asset URLs.
 *
 * Handles:
 *  - `![alt](relative/path)` → `![alt](http://asset.localhost/...)`
 *  - `<video src="relative/path">` → `<video src="http://asset.localhost/...">`
 */
export function relativePathToAssetUrl(markdown: string, workspacePath: string): string {
  // Normalize workspace path separators
  const wsPath = workspacePath.replace(/\\/g, "/");

  // Match markdown image/video: ![...](url) — skip absolute URLs and data: URIs
  const mdPattern = /(!\[[^\]]*]\()([^)]+)(\))/g;
  let result = markdown.replace(mdPattern, (_match, prefix, url, suffix) => {
    if (isAbsoluteOrSpecialUrl(url)) return `${prefix}${url}${suffix}`;
    const absPath = `${wsPath}/${url}`;
    return `${prefix}${convertFileSrc(absPath)}${suffix}`;
  });

  // Match HTML src attributes: src="..." — for <video>, <img> etc.
  const htmlPattern = /(src=")([^"]+)(")/g;
  result = result.replace(htmlPattern, (_match, prefix, url, suffix) => {
    if (isAbsoluteOrSpecialUrl(url)) return `${prefix}${url}${suffix}`;
    const absPath = `${wsPath}/${url}`;
    return `${prefix}${convertFileSrc(absPath)}${suffix}`;
  });

  return result;
}

/**
 * Post-process markdown on save: convert asset URLs back to relative paths.
 */
export function assetUrlToRelativePath(markdown: string, workspacePath: string): string {
  // Normalize workspace path for matching
  const wsPath = workspacePath.replace(/\\/g, "/");

  // Build the expected asset URL prefix for this workspace
  const wsAssetPrefix = convertFileSrc(`${wsPath}/`);

  // Replace all occurrences of the workspace asset URL prefix with empty string (= relative path)
  return markdown.split(wsAssetPrefix).join("");
}

function isAbsoluteOrSpecialUrl(url: string): boolean {
  return (
    url.startsWith("http://") ||
    url.startsWith("https://") ||
    url.startsWith("data:") ||
    url.startsWith("blob:") ||
    url.startsWith("/")
  );
}
