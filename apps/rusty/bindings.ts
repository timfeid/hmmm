// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "version", input: never, result: string },
    mutations: 
        { key: "authentication.login", input: LoginArgs, result: AuthResponse } | 
        { key: "authentication.refresh_token", input: string, result: AuthResponse } | 
        { key: "lobby.action", input: LobbyActionArgs, result: null } | 
        { key: "lobby.create", input: string[], result: LobbyData } | 
        { key: "lobby.input", input: LobbyInputArgs, result: null } | 
        { key: "lobby.join", input: string, result: null } | 
        { key: "lobby.ready", input: string, result: null },
    subscriptions: 
        { key: "lobby.subscribe", input: [string, string], result: PersonalizedGameData }
};

export type ActionTrigger = { trigger_type: ActionTriggerType }

export type Coordinates = { x: number; y: number }

export type AuthResponse = { access_token: string | null; refresh_token: string | null; success: boolean }

export type LoginArgs = { username: string; password: string }

export type LobbyData = { join_code: string; chat: LobbyChat[] }

export type LobbyInputArgs = { access_token: string; lobby_id: string; r: number; x: number; y: number }

export type PersonSkin = "Default"

export type LobbyActionArgs = { access_token: string; lobby_id: string; action_id: string }

export type PersonalizedGameData = { visible_objects: { [key: string]: OutgoingGameObject } }

export type LobbyChat = { user_id: string; message: string }

export type ActionTriggerType = { ActionKeyPressed: number }

export type CarDetails = { skin: CarSkin; speed: number; acceleration: number; max_passengers: number; passenger_user_ids: string[]; rotation_speed: number; driver_user_id: string | null }

export type CarSkin = "Sedan" | "Police"

export type PersonDetails = { user_id: string; skin: PersonSkin }

export type OutgoingGameObject = { id: string; x: number; y: number; rotation: number; velocity: Coordinates; owner_user_id: string; controller_user_id: string | null; details: GameObjectInfo; action: ActionTrigger | null }

export type GameObjectInfo = { Person: PersonDetails } | { Car: CarDetails }
