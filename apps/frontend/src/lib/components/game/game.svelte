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
    roadLayer!: Phaser.Tilemaps.TilemapLayer;
    private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
    private cars: Map<String, Car> = new Map();
    private carGroup!: Phaser.Physics.Arcade.Group;

    private controller!: PlayerController;
    private action!: Phaser.Input.Keyboard.Key;
    private actionables: Array<any> = [];
    private tileSize: number = 16;

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

    setNighttime(w: number, h: number) {
      const nightOverlay = this.add.rectangle(
        0,
        0,
        w,
        h,
        0x000000,
        0.6 // alpha value: 0.6 gives a dark overlay
      );
      nightOverlay.setOrigin(0, 0);
      // Set the overlay's depth to be on top of everything.
      nightOverlay.setDepth(10);
      nightOverlay.setPipeline("Light2D");

      // Optionally, animate the overlay to simulate a day/night cycle:
      // this.tweens.add({
      //   targets: nightOverlay,
      //   alpha: 0.2, // daylight
      //   duration: 15000,
      //   yoyo: true,
      //   repeat: -1,
      //   ease: "Sine.easeInOut",
      // });
    }

    create() {
      this.carGroup = this.physics.add.group();
      this.lights.enable().setAmbientColor(0xffffff); // dark ambient light
      this.physics.add.collider(
        this.carGroup,
        this.carGroup,
        (spriteA, spriteB) => {
          // This callback is called when two car sprites collide.
          console.log("Car collision between:", spriteA, spriteB);
          // You can add additional collision logic here.
        }
      );

      const map = this.make.tilemap({ key: "map" });
      const tileset = map.addTilesetImage(
        "0x72-industrial-tileset-32px-extruded",
        "tiles"
      );
      if (!tileset) {
        console.error("Tileset not found!");
        return;
      }
      const groundLayer = map.createLayer("ground", tileset, 0, 0);
      groundLayer?.setOrigin(0, 0);
      const roadLayer = map.createLayer("road", tileset, 64, 208);
      // roadLayer?.setOrigin(0, 0);

      if (!groundLayer || !roadLayer) {
        console.error("Layer not found!");
        return;
      }
      groundLayer.setPipeline("Light2D");
      roadLayer.setPipeline("Light2D");
      groundLayer.setDepth(0);
      this.roadLayer = roadLayer;
      this.tileSize = map.tileWidth;

      let minX = Number.MAX_VALUE;
      let minY = Number.MAX_VALUE;
      let maxX = 0;
      let maxY = 0;

      groundLayer.forEachTile((tile) => {
        // tile.index of -1 means empty in Phaser.
        if (tile.index !== -1) {
          if (tile.pixelX < minX) {
            minX = tile.pixelX;
          }
          if (tile.pixelY < minY) {
            minY = tile.pixelY;
          }
          if (tile.pixelX + tile.width > maxX) {
            maxX = tile.pixelX + tile.width;
          }
          if (tile.pixelY + tile.height > maxY) {
            maxY = tile.pixelY + tile.height;
          }
        }
      });

      console.log("Ground tile bounds:", minX, minY, maxX, maxY);

      // Set world and camera bounds to match the region where actual ground tiles exist.
      this.physics.world.setBounds(minX, minY, maxX - minX, maxY - minY);
      this.cameras.main.setBounds(minX, minY, maxX - minX, maxY - minY);
      this.setNighttime(maxX, maxY);

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
      const carSprite = this.physics.add.sprite(490, 677, "car-north");
      carSprite.setDisplaySize(this.tileSize * 2, this.tileSize * 2);
      carSprite.body.setSize(this.tileSize * 2, this.tileSize * 2);
      carSprite.setDepth(1);
      carSprite.setCollideWorldBounds(true);
      const car = new Car(user.user!.sub, this, carSprite, 220, 5);
      this.actionables.push(car);
      this.cars.set(user.user!.sub, car);

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

      const personSprite = this.physics.add.sprite(485, 752, "person-idle", 0);
      personSprite.anims.play("idle");

      personSprite.setDisplaySize(this.tileSize, this.tileSize);
      personSprite.body.setSize(this.tileSize, this.tileSize);
      personSprite.setDepth(1);
      personSprite.setCollideWorldBounds(true);
      const person = new Person(user.user!.sub, personSprite);
      this.actionables.push(person);

      // Start with the person as the controlled entity.
      this.controller = new PlayerController(user.user!.sub, person);

      // Subscribe to server state updates as needed.
    }

    updateCars() {
      for (const [userId, u] of Object.entries(
        lobby?.game_state.visible_users || {}
      )) {
        if (!this.cars.has(userId)) {
          const carSprite = this.physics.add.sprite(490, 677, "car-north");
          carSprite.setDisplaySize(this.tileSize * 2, this.tileSize * 2);
          carSprite.body.setSize(this.tileSize * 2, this.tileSize * 2);
          carSprite.setDepth(1);
          carSprite.setCollideWorldBounds(true);
          const car = new Car(userId, this, carSprite, 220, 5);
          this.cars.set(userId, car);
        }

        if (userId != user.user!.sub) {
          this.cars.get(userId)!.getSprite().setPosition(u.x, u.y);
          this.cars.get(userId)!.getSprite().setRotation(u.rotation);
        }
        // this.cars.get(userId)!.updateInput({
        //   u
        // }, 0);
      }
    }

    update(time: number, delta: number) {
      // Let the current controlled entity update its movement based on input.
      console.log(this.lights.getLights(this.cameras.main));
      this.controller.update(this.cursors, delta);
      this.updateCars();

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
      const pos = this.controller.getSprite().body?.position;
      // if (pos) {
      //   console.log(pos);
      // }
      // this.cameras.main.setBounds(0, 0, 1600, 1200);
      this.cameras.main.startFollow(
        this.controller.getSprite(),
        true,
        0.08,
        0.08
      );
      this.cameras.main.setDeadzone(100, 100);
    }
  }

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
      game = new Phaser.Game({
        type: Phaser.WEBGL,
        scene: Example,
        physics: {
          default: "arcade",
          arcade: { gravity: { y: 0, x: 0 } },
        },
      });
    }
  });
</script>

{#if !game}
  <Button href="/">Home?</Button>
{/if}
<div class="text-2xl">Hello {user.user?.sub || ""}</div>
