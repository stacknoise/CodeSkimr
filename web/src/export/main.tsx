import "../shared/theme/tokens.css";
import "../app/global.css";
import { render } from "solid-js/web";
import { ROOT_ID } from "../app/consts";
import { currentLanguage, setLanguage } from "../shared/i18n/i18n";
import { openProject } from "../shared/project/active";
import { applyTheme } from "../shared/theme/theme";
import { installData } from "./commands";
import { ExportApp } from "./ExportApp";
import { readData } from "./read";
import { TITLE_SEPARATOR } from "./consts";

const root = document.getElementById(ROOT_ID);
const data = readData();
if (root && data) {
  applyTheme();
  setLanguage(currentLanguage());
  installData(data);
  openProject(data.project.id);
  document.title = `${data.project.name}${TITLE_SEPARATOR}${document.title}`;
  render(() => <ExportApp data={data} />, root);
}
