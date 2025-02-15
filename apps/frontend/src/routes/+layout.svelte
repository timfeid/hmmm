<script lang="ts">
  import "../app.css";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { createTauriListeners } from "../lib/tauri";
  import { onMount } from "svelte";
  import { getAccessToken } from "../lib/auth";
  import { user } from "../lib/stores/access-token.svelte";

  let loading = true;

  onMount(async () => {
    try {
      const at = await getAccessToken();
      user.accessToken = at || undefined;
    } catch (e) {
      console.log(e);
    }
    loading = false;
  });

  onMount(createTauriListeners);

  let { children } = $props();
</script>

<!-- <pre>{JSON.stringify(user.user, null, 2)}</pre> -->
{@render children()}

<Toaster />
