// Picks the language of the loading page before the app is there to do it.
(function () {
  var german = (navigator.language || "").toLowerCase().indexOf("de") === 0;
  document.documentElement.setAttribute("lang", german ? "de" : "en");
})();
