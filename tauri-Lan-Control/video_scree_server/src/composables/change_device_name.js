import { ElNotification } from "element-plus";
import { ref, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";

export const change_deive_name = () => {
  const setEdit = ref({});
  const inputRefs = ref([]);

  const setItemRef = (index) => {
    return (el) => {
      inputRefs.value[index] = el;
    };
  };
  // 失去焦点 / 回车执行修改名称
  const checkNameChange = async (item) => {
    const newName = setEdit.value[item.device_ip]?.newName;
    if (item.device_name === newName || !newName) {
      setEdit.value[item.device_ip].state = false;
      ElNotification({
        type: "warning",
        title: "tips",
        message: "未修改名称",
        duration: 3000,
      });
      return;
    }

    item.device_name = newName;
    setEdit.value[item.device_ip].state = false;
    // 修改名称
    await invoke("sned_fn", {
      ip: item.device_ip,
      key: `change_name|${newName}`,
    });
    ElNotification({
      type: "success",
      title: "tips",
      message: "已修改名称",
      duration: 3000,
    });
  };

  // 开启编辑名称
  const enableEdit = (item, index) => {
    ElNotification({
      type: "info",
      title: "编辑名称",
      message: "使用回车/空白处点击即可应用新名称",
      duration: 3000,
    });
    setEdit.value[item.device_ip] = {
      state: true,
      newName: "",
    };
    nextTick(() => {
      inputRefs.value[index]?.focus();
    });
  };

  return {
    setEdit,
    inputRefs,
    setItemRef,
    checkNameChange,
    enableEdit,
  };
};