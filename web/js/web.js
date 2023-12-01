import { default as collection } from "./collection";
import CodeMirror from "codemirror/lib/codemirror.js";
import { dump } from "js-yaml";
import * as postman2openapi from "postman2openapi";
import _CodeMirrorStyles from "../css/codemirror.css";
import _DemoStyles from "../css/demo.css";

const openApiCopyBtn = document.getElementById("openapi-copy-btn");

const postmanElement = CodeMirror.fromTextArea(
  document.getElementById("postman"),
  {
    lineNumbers: true,
  },
);

postmanElement.setValue(JSON.stringify(collection, 0, 2));

postmanElement.on("change", (_) => {
  update();
});

const openapiElement = CodeMirror.fromTextArea(
  document.getElementById("openapi"),
  {
    readOnly: true,
  },
);

const update = () => {
  const postman = postmanElement.getValue();
  try {
    const openapi = postman2openapi.transpile(JSON.parse(postman));
    console.log(openapi);
    const output = dump(
      openapi instanceof Map ? Object.fromEntries(openapi) : openapi,
      {
        replacer: (_key, value) => {
          if (value instanceof Map) {
            return Object.fromEntries(value);
          }

          return value;
        },
      },
    );
    openapiElement.setValue(output);
  } catch (e) {
    openapiElement.setValue(e);
  }
};

openApiCopyBtn.addEventListener("click", (e) => {
  if (window && window.navigator) {
    navigator.clipboard.writeText(openapiElement.getValue()).then(() => {
      openApiCopyBtn.innerText = "Copied";
    });
  }
});

update();
