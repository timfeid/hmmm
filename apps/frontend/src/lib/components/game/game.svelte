<script lang="ts">
  import type { LobbyCommand, LobbyData } from "@gangsta/rusty";
  import Phaser from "phaser";
  import { onMount } from "svelte";
  import { client, websocketClient } from "../../client";
  import { user } from "../../stores/access-token.svelte";
  import Button from "../ui/button/button.svelte";
  import { PlayerController } from "./player-controller";
  import { Person } from "./person.js";
  import { Car } from "./car.js";

  class Example extends Phaser.Scene {
    private roadLayer!: Phaser.Tilemaps.TilemapLayer;
    private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
    private controller!: PlayerController;
    private action!: Phaser.Input.Keyboard.Key;
    private actionables: Array<any> = [];

    preload() {
      this.load.tilemapTiledJSON("map", "/assets/map.json");
      this.load.image("tiles", "/assets/tiles.png");
      this.load.image("car-north", "/assets/car/car-north.png");
      this.load.image("car-south", "/assets/car/car-south.png");
      // this.load.image("person-idle", "/assets/person/idle.png");
      this.load.spritesheet("person-run", "/assets/person/run.png", {
        frameWidth: 16,
        frameHeight: 16,
        spacing: 64,
        margin: 32,
      });
      this.load.spritesheet("person-walk", "/assets/person/walk.png", {
        frameWidth: 16,
        frameHeight: 16,
        spacing: 64,
        margin: 32,
      });
      this.load.spritesheet("person-idle", "/assets/person/idle.png", {
        frameWidth: 16,
        frameHeight: 16,
        spacing: 64,
        margin: 32,
      });
    }

    create() {
      const map = this.make.tilemap({ key: "map" });
      const tileset = map.addTilesetImage(
        "0x72-industrial-tileset-32px-extruded",
        "tiles"
      );
      if (!tileset) {
        console.error("Tileset not found!");
        return;
      }
      const groundLayer = map.createLayer("Ground", tileset, 0, 0);
      const roadLayer = map.createLayer("Road", tileset, 64, -304);
      if (!groundLayer || !roadLayer) {
        console.error("Layer not found!");
        return;
      }
      groundLayer.setDepth(0);
      this.roadLayer = roadLayer;
      const tileSize = map.tileWidth;

      // Create WASD keys.
      this.cursors = this.input.keyboard.addKeys({
        up: Phaser.Input.Keyboard.KeyCodes.W,
        left: Phaser.Input.Keyboard.KeyCodes.A,
        down: Phaser.Input.Keyboard.KeyCodes.S,
        right: Phaser.Input.Keyboard.KeyCodes.D,
      }) as Phaser.Types.Input.Keyboard.CursorKeys;
      // Key to enter/exit car, e.g., E.
      this.action = this.input.keyboard.addKey(
        Phaser.Input.Keyboard.KeyCodes.E
      );

      // Create a car entity.
      const carSprite = this.physics.add.sprite(
        tileSize * 20,
        tileSize * 21,
        "car-north"
      );
      carSprite.setDisplaySize(tileSize * 2, tileSize * 2);
      carSprite.body.setSize(tileSize * 2, tileSize * 2);
      carSprite.setDepth(1);
      carSprite.setCollideWorldBounds(true);
      const car = new Car("car1", roadLayer, carSprite, 100, 5);
      this.actionables.push(car);

      this.anims.create({
        key: "walk",
        frames: this.anims.generateFrameNumbers("person-walk", {
          start: 0,
          end: 7,
        }), // adjust range as needed
        frameRate: 8,
        repeat: -1,
      });
      this.anims.create({
        key: "idle",
        frames: this.anims.generateFrameNumbers("person-idle", {
          start: 0,
          end: 3,
        }), // adjust range as needed
        frameRate: 8,
        repeat: -1,
      });
      // Create a person entity.

      const personSprite = this.physics.add.sprite(0, 0, "person-idle", 0);
      personSprite.anims.play("idle");

      personSprite.setDisplaySize(tileSize, tileSize);
      personSprite.body.setSize(tileSize, tileSize);
      personSprite.setDepth(1);
      personSprite.setCollideWorldBounds(true);
      const person = new Person(user.user!.sub, personSprite);
      this.actionables.push(person);

      // Start with the person as the controlled entity.
      this.controller = new PlayerController(user.user!.sub, person);

      // Subscribe to server state updates as needed.
    }

    update(time: number, delta: number) {
      // Let the current controlled entity update its movement based on input.
      this.controller.update(this.cursors, delta);

      // Example: Press E to toggle between person and car.

      if (Phaser.Input.Keyboard.JustDown(this.action)) {
        this.controller.action(this.actionables);
        // If currently controlling person, and near the car, switch control.
        // Otherwise, if controlling the car, switch back to person.
        // (Distance check and transfer logic goes here)
      }

      // Broadcast input state:
      const inputState = {
        lobby_id: gameId,
        access_token: user.accessToken!,
        ...this.controller.getInputState(),
      };
      // websocketClient.mutation(["lobby.input", inputState]);
      this.cameras.main.startFollow(
        this.controller.getSprite(),
        true,
        0.08,
        0.08
      );
      this.cameras.main.setDeadzone(100, 100);
    }
  }

  const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    scene: Example,
    physics: {
      default: "arcade",
      arcade: { gravity: { y: 0 } },
    },
  };

  function isUpdated(data: LobbyCommand): data is { Updated: LobbyData } {
    return "Updated" in data;
  }

  function updated(data: LobbyData) {
    // console.log(data.game_state.visible_users);
    lobby = data;
  }

  function onData(data: LobbyCommand) {
    if (isUpdated(data)) {
      return updated(data.Updated);
    }
  }

  let { gameId } = $props();

  let lobby = $state<undefined | LobbyData>();
  let game: Phaser.Game | undefined = $state();
  onMount(() => {
    let unsubscribe: (() => void) | undefined;
    if (user.accessToken) {
      console.log(["lobby.subscribe", [gameId, user.accessToken]]);
      unsubscribe = websocketClient.addSubscription(
        ["lobby.subscribe", [gameId, user.accessToken]],
        {
          onData,
        }
      );
    }

    return () => {
      game?.destroy(true);
      if (unsubscribe) {
        unsubscribe();
      }
    };
  });

  $effect(() => {
    if (!game && lobby) {
      game = new Phaser.Game(config);
    }
  });
</script>

{#if !game}
  <Button href="/">Home?</Button>
{/if}
<div class="text-2xl">Hello {user.user?.sub || ""}</div>
