<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Game from "../lib/components/game/game.svelte";
  import { Button } from "../lib/components/ui/button";
  import { client } from "../lib/client";
  import { user } from "../lib/stores/access-token.svelte";
  import { goto } from "$app/navigation";

  let name = $state("");
  let greetMsg = $state("");
  let gameId = $state<string | undefined>(undefined);

  async function createGame() {
    const response = await client.mutation(["lobby.create", []]);
    goto(`game/${response.join_code}`);
  }

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container mx-auto">
  {#if !user.accessToken}
    <Button href="/login">Log in</Button>
  {:else}
    <Button onclick={createGame}>Create a game</Button>
  {/if}
</main>
