import { Store } from "@tauri-apps/plugin-store";
import { useAppConfig } from "@/store/config";

export const init_store = async () => {
  const AppConfig = useAppConfig();
  // 加载json配置
  const store = await Store.load(".server_settings.json");
  const store_config = await store.get("config");
  const store_script = await store.get("script");
  // 脚本
  if (store_script) {
    AppConfig.option_script.length = 0;
    AppConfig.option_script = store_script;
  }
  // 配置
  if (store_config) {
    AppConfig.config = store_config;
  } else {
    await store.set("config", AppConfig.config);
    await store.save();
  }
};
