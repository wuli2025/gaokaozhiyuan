import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./style.css";
import "./styles/festive.css";
import { initTheme } from "./stores/theme";

// 在挂载前写入主题（默认「鎏金朱砂·喜庆」），避免首屏主题闪烁
initTheme();

const app = createApp(App);
app.use(createPinia());
app.mount("#app");
