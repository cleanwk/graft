import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { applyStoredTheme } from "./lib/theme";
import "./styles/tokens.css";
import "./styles/global.css";

applyStoredTheme();
createApp(App).use(createPinia()).mount("#app");
