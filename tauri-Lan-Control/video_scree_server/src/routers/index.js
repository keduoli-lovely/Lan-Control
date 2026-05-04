import { createMemoryHistory, createRouter } from "vue-router";

import setting from "@/Components/setting.vue";
import setting_script from "@/Components/setting_script.vue";
import setting_script_add from "@/Components/setting_script_add.vue";

const routes = [
  { path: "/", component: setting },
  { path: "/select", component: setting_script },
  { path: "/add", component: setting_script_add },
];

const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

export default router
