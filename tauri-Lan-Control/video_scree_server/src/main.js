import { createApp } from "vue";
import App from "./App.vue";
import { createPinia } from "pinia";
import router from "./routers";
import "element-plus/dist/index.css";
import "./css/global.css";
import "./eventBus/bus";

const pinia = createPinia();
const app = createApp(App);

app.use(pinia).use(router).mount("#app");
