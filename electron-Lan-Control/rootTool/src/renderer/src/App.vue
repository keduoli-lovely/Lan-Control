<script setup>
import Home from './components/Home.vue'
import { initListeners } from './utils/popupStore';
import { onMounted } from 'vue';

onMounted(() => {
    initListeners();
});
</script>

<template>
    <div>
        <Home />
    </div>
</template>


<style lang="scss">
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

::-webkit-scrollbar {
    width: 0px;
    background: transparent;
}
</style>