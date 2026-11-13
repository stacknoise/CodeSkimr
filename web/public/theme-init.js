// Generated from design/tokens.toml by tools/codegen. Do not edit.
(function () {
  var themes = ["hell", "petrol", "dunkel"];
  var chosen = null;
  try {
    chosen = localStorage.getItem("codeskimr.theme");
  } catch (error) {
    chosen = null;
  }
  var dark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
  var fallback = dark ? "dunkel" : "petrol";
  var theme = themes.indexOf(chosen) >= 0 ? chosen : fallback;
  document.documentElement.setAttribute("data-theme", theme);
})();
