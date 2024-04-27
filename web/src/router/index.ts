import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      component: () => import("@/pages/MainView.vue"),
    },
    {
      path: "/debug",
      component: () => import("@/pages/DebugView.vue"),
    },
  ],
});

export default router;
