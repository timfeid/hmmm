<script lang="ts">
  import { page } from "$app/state";
  import Game from "$lib/components/game/game.svelte";
  import { Button } from "../../../lib/components/ui/button";
  import { user } from "../../../lib/stores/access-token.svelte";

  let gameId = page.params.slug;
</script>

<main class="container mx-auto">
  {#if user.accessToken && gameId}
    <Game {gameId}></Game>
  {:else}
    <Button href="/login">Log in</Button>
  {/if}
</main>
