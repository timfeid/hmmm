<script lang="ts">
  import type {
    OutgoingGameObject,
    PersonalizedGameData,
  } from "@gangsta/rusty";
  import Phaser from "phaser";
  import { onMount } from "svelte";
  import { client, websocketClient } from "../../client";
  import { user } from "../../stores/access-token.svelte";
  import Button from "../ui/button/button.svelte";
  import { PlayerController } from "./player-controller";
  import { Person } from "./person.js";
  import { Car } from "./car.js";
  import { debounce, throttle } from "../../utils";
  import { toast } from "svelte-sonner";
  import type { ServerUpdatable } from "./updatable";
  import type { Controllable } from "./controllable";
  import { isCar, isPerson, type CarObject, type PersonObject } from "./utils";
  import type { Actionable } from "./actionable";

  const userId = $derived(user.user?.sub || "");

  class Example extends Phaser.Scene {
    roadLayer!: Phaser.Tilemaps.TilemapLayer;
    private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
    private objects: Map<String, ServerUpdatable> = new Map();
    carGroup!: Phaser.Physics.Arcade.Group;

    private controller?: PlayerController;
    private action!: Phaser.Input.Keyboard.Key;
    private actionables: Array<Actionable> = [];
    private tileSize: number = 16;
    private tilePosText!: Phaser.OutgoingGameObjects.Text;

    personGroup!: Phaser.Physics.Arcade.Group;

    preload() {
      this.load.tilemapTiledJSON("map", "/assets/map.json");
      this.load.image("tiles", "/assets/tiles.png");
      this.load.image("Sedan", "/assets/car/car.png");
      this.load.image("Police", "/assets/car/police.png");

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
      const nightOverlay = this.add.rectangle(0, 0, w, h, 0x000000, 0.6);
      nightOverlay.setOrigin(0, 0);
      nightOverlay.setDepth(10);
      nightOverlay.setPipeline("Light2D");
    }

    createDebugBox() {
      this.tilePosText = this.add.text(0, 0, "Tile: (0, 0)", {
        fontSize: "16px",
        color: "#ffffff",
        backgroundColor: "#00000080",
      });
      this.tilePosText.setScrollFactor(0);
      this.tilePosText.setOrigin(1, 0);

      this.tilePosText.setPosition(this.cameras.main.width - 10, 10);
      this.tilePosText.setDepth(400);
    }

    create() {
      this.carGroup = this.physics.add.group();
      this.createDebugBox();
      this.personGroup = this.physics.add.group();
      this.lights.enable().setAmbientColor(0xcccccc);
      this.physics.add.collider(
        this.carGroup,
        this.personGroup,
        (spriteA, spriteB) => {}
      );
      this.physics.add.collider(
        this.carGroup,
        this.carGroup,
        (spriteA, spriteB) => {
          console.log("Car/car collision between:", spriteA, spriteB);
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
      const roadLayer = map.createLayer("road", tileset, 0, 0);

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

      this.physics.world.setBounds(minX, minY, maxX - minX, maxY - minY);
      this.cameras.main.setBounds(minX, minY, maxX - minX, maxY - minY);

      this.cursors = this.input.keyboard.addKeys({
        up: Phaser.Input.Keyboard.KeyCodes.W,
        left: Phaser.Input.Keyboard.KeyCodes.A,
        down: Phaser.Input.Keyboard.KeyCodes.S,
        right: Phaser.Input.Keyboard.KeyCodes.D,
      }) as Phaser.Types.Input.Keyboard.CursorKeys;

      this.action = this.input.keyboard.addKey(
        Phaser.Input.Keyboard.KeyCodes.E
      );

      this.anims.create({
        key: "walk",
        frames: this.anims.generateFrameNumbers("person-walk", {
          start: 0,
          end: 7,
        }),
        frameRate: 8,
        repeat: -1,
      });
      this.anims.create({
        key: "idle",
        frames: this.anims.generateFrameNumbers("person-idle", {
          start: 0,
          end: 3,
        }),
        frameRate: 8,
        repeat: -1,
      });
    }

    createPerson(object: PersonObject) {
      const person = new Person(object, this);
      this.actionables.push(person);

      if (object.details.Person.user_id === userId) {
        this.controller = new PlayerController(user.user!.sub, person);
        this.controller.addEventListener(
          "updated",
          (e: CustomEvent<{ entity: Controllable }>) => {}
        );
      }

      return person;
    }

    createCar(object: CarObject) {
      const car = new Car(object, this);
      this.actionables.push(car);

      return car;
    }

    createObject(object: OutgoingGameObject) {
      if (isCar(object)) {
        return this.createCar(object);
      }

      if (isPerson(object)) {
        return this.createPerson(object);
      }

      console.error(object);
      throw new Error("Unknown object" + object);
    }

    updateDebugBox() {
      if (!this.controller) {
        return;
      }
      const sprite = this.controller.getControlledEntity().sprite;
      const tileX = Math.floor(sprite.x / this.tileSize);
      const tileY = Math.floor(sprite.y / this.tileSize);

      this.tilePosText.setText(`Tile: (${tileX}, ${tileY})`);
    }

    updateObjects(time: number, delta: number) {
      for (const [objectId, object] of Object.entries(
        lobby?.visible_objects || {}
      )) {
        if (!this.objects.has(objectId)) {
          try {
            const updateable = this.createObject(object);
            this.objects.set(objectId, updateable);
          } catch (e) {
            toast.error((e as Error).message);
          }
        }

        this.objects.get(objectId)!.updateInputFromServer(object, time, delta);
      }
    }

    async performUpdate(inputState: any) {
      await websocketClient.mutation(["lobby.input", inputState]);
    }

    throttledUpdate = throttle((input: any) => {
      this.performUpdate(input);
    });

    getInputState(entity: Controllable) {
      return {
        lobby_id: gameId,
        access_token: user.accessToken!,
        object_id: entity.id,
        ...entity.getInputState(),
      };
    }

    updateServer(entity: Controllable, throttle = true) {
      if (!this.controller) {
        return;
      }

      const inputState = this.getInputState(entity);
      if (throttle) {
        return this.throttledUpdate(inputState, 150);
      }

      return this.performUpdate(inputState);
    }

    update(time: number, delta: number) {
      this.updateObjects(time, delta);
      this.updateDebugBox();

      if (!this.controller) {
        console.log("oh no controller");
        return;
      }
      this.controller.update(this.cursors, delta);
      if (Phaser.Input.Keyboard.JustDown(this.action)) {
        this.performAction();
      }
      this.updateServer(this.controller.getControlledEntity());
    }

    async executeAction(actionId: string) {
      return await websocketClient.mutation([
        "lobby.action",
        {
          lobby_id: gameId,
          access_token: user.accessToken!,
          action_id: actionId,
        },
      ]);
    }

    async performAction() {
      if (!this.controller || !lobby) {
        console.log("oh no controller");
        return;
      }

      const action = this.controller.action(
        Object.values(lobby.visible_objects)
      );
      if (action) {
        try {
          const response = await this.executeAction(action.id);
          await this.updateServer(this.controller.getControlledEntity(), false);

          this.actionables
            .find((a) => a.id === action.id)
            ?.action(this.controller);

          return response;
        } catch (e) {
          toast.error((e as Error).message);
          return false;
        }
      }
    }
  }

  function onData(data: PersonalizedGameData) {
    lobby = data;
  }

  let { gameId } = $props();

  let lobby = $state<undefined | PersonalizedGameData>();
  let game: Phaser.Game | undefined = $state();
  onMount(() => {
    let unsubscribe: (() => void) | undefined;
    if (user.accessToken) {
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
  const width = 1920;
  const height = 1080;

  $effect(() => {
    if (!game && lobby) {
      game = new Phaser.Game({
        type: Phaser.WEBGL,
        scene: Example,
        physics: {
          default: "arcade",
          arcade: { gravity: { y: 0, x: 0 } },
        },
        width,
        height,
      });
    }
  });
</script>

{#if !game}
  <Button href="/">Home?</Button>
{/if}
<div class="text-2xl">Hello {user.user?.sub || ""}</div>
