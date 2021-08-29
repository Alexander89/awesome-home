import { Fish, FishId, Tag } from '@actyx/pond'
import { Emitter } from './types'

export type UndefinedState = {
  state: 'undefined'
  id: string
}
export type ReadyState = {
  state: 'ready'
  id: string
  ip: string
  ssid: string
  battery: number
}
export type ConnectedState = {
  state: 'connected'
  id: string
  ip: string
  ssid: string
  battery: number
}
export type LaunchedState = {
  state: 'launched'
  id: string
  ip: string
  ssid: string
  missionId: string
  atWaypointId: number
  targetWaypointId?: number
  battery: number
}
export type UsedState = {
  state: 'used'
  id: string
  ip: string
  ssid: string
  lastMissionId: string
  battery: number
}
export type DroneState = UndefinedState | ReadyState | ConnectedState | LaunchedState | UsedState

/*
droneReady(id: string, ip: string)
droneConnected(id: string)
droneStatsUpdated(id: string, battery: number)
droneLaunched(id: string, missionId: string)
droneStartedToNextWaypoint(id: string, missionId: string, waypointId: number)
droneArrivedAtWaypoint(id: string, missionId: string, waypointId: number)
droneMissionCompleted(id: string, missionId: string)
droneLanded(id: string, at: { x: number, y: number, z: number})
droneDisconnected(id: string)
*/
export type DroneDefinedEvent = {
  eventType: 'droneDefined'
  id: string
  ssid: string
  ip: string
}
export type DroneReadyEvent = {
  eventType: 'droneReady'
  id: string
}
export type DroneConnectedEvent = {
  eventType: 'droneConnected'
  id: string
}
export type DroneStatsUpdatedEvent = {
  eventType: 'droneStatsUpdated'
  id: string
  battery: number
}
export type DroneLaunchedEvent = {
  eventType: 'droneLaunched'
  id: string
  missionId: string
}
export type DroneStartedToNextWaypointEvent = {
  eventType: 'droneStartedToNextWaypoint'
  id: string
  missionId: string
  waypointId: number
}
export type DroneArrivedAtWaypointEvent = {
  eventType: 'droneArrivedAtWaypoint'
  id: string
  missionId: string
  waypointId: number
}
export type DroneMissionCompletedEvent = {
  eventType: 'droneMissionCompleted'
  id: string
  missionId: string
}
export type DroneLandedEvent = {
  eventType: 'droneLanded'
  id: string
  at: { x: number; y: number; z: number }
}
export type DroneDisconnectedEvent = {
  eventType: 'droneDisconnected'
  id: string
}
export type DroneEvent =
  | DroneDefinedEvent
  | DroneReadyEvent
  | DroneConnectedEvent
  | DroneStatsUpdatedEvent
  | DroneLaunchedEvent
  | DroneStartedToNextWaypointEvent
  | DroneArrivedAtWaypointEvent
  | DroneMissionCompletedEvent
  | DroneLandedEvent
  | DroneDisconnectedEvent

const emitDroneDefined: Emitter<DroneDefinedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id).and(droneDefinedTag), { eventType: 'droneDefined', ...event })

const emitDroneReady: Emitter<DroneReadyEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneReady', ...event })

const emitDroneConnected: Emitter<DroneConnectedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneConnected', ...event })

const emitDroneStatsUpdated: Emitter<DroneStatsUpdatedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneStatsUpdated', ...event })

const emitDroneLaunched: Emitter<DroneLaunchedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id).and(droneMissionStartedTag), {
    eventType: 'droneLaunched',
    ...event,
  })

const emitDroneStartedToNextWaypoint: Emitter<DroneStartedToNextWaypointEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneStartedToNextWaypoint', ...event })

const emitDroneArrivedAtWaypoint: Emitter<DroneArrivedAtWaypointEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneArrivedAtWaypoint', ...event })

const emitDroneMissionCompleted: Emitter<DroneMissionCompletedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id).and(droneMissionCompletedTag), {
    eventType: 'droneMissionCompleted',
    ...event,
  })

const emitDroneLanded: Emitter<DroneLandedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneLanded', ...event })

const emitDroneDisconnected: Emitter<DroneDisconnectedEvent> = (emit, event) =>
  emit(droneTag.withId(event.id), { eventType: 'droneDisconnected', ...event })

const droneTag = Tag<DroneEvent>('drone')
const droneDefinedTag = Tag<DroneDefinedEvent>('drone.defined')
const droneMissionStartedTag = Tag<DroneLaunchedEvent>('drone.mission.started')
const droneMissionCompletedTag = Tag<DroneMissionCompletedEvent>('drone.mission.completed')

export const DroneTwins = {
  // Tags
  tags: {
    droneTag,
    droneDefinedTag,
    droneMissionStartedTag,
    droneMissionCompletedTag,
  },
  // Twins
  of: (id: string): Fish<DroneState, DroneEvent> => ({
    fishId: FishId.of('com.awesome-home.launchpad', id, 0),
    initialState: { state: 'undefined', id },
    where: droneTag.withId(id),
    onEvent: (state, event) => {
      if (state.state === 'undefined') {
        if (event.eventType === 'droneDefined') {
          return {
            state: 'ready',
            id,
            ip: event.ip,
            ssid: event.ssid,
            battery: 0,
          }
        }
        return state
      }

      if (event.eventType === 'droneStatsUpdated') {
        state.battery = event.battery
        return state
      }

      switch (state.state) {
        case 'ready':
          switch (event.eventType) {
            case 'droneConnected':
              return {
                ...state,
                state: 'connected',
              }

            default:
              console.log(`Never reach: ${event.eventType} in 'ready' state`)
              return state
          }
        case 'connected':
          switch (event.eventType) {
            case 'droneLaunched':
              return {
                ...state,
                state: 'launched',
                missionId: event.missionId,
                atWaypointId: 0,
                targetWaypointId: undefined,
              }
            case 'droneDisconnected':
              return {
                ...state,
                state: 'ready',
              }
            default:
              console.log(`Never reach: ${event.eventType} in 'connected' state`)
              return state
          }
        case 'launched':
          switch (event.eventType) {
            case 'droneStartedToNextWaypoint':
              return {
                ...state,
                state: 'launched',
                targetWaypointId: event.waypointId,
              }
            case 'droneArrivedAtWaypoint':
              return {
                ...state,
                state: 'launched',
                atWaypointId: event.waypointId,
                targetWaypointId: undefined,
              }
            case 'droneLanded':
              return {
                state: 'used',
                id,
                ip: state.ip,
                ssid: state.ssid,
                battery: state.battery,
                lastMissionId: state.missionId,
              }
            case 'droneReady':
              return {
                state: 'ready',
                id,
                ip: state.ip,
                ssid: state.ssid,
                battery: state.battery,
              }
            default:
              console.log(`Never reach: ${event.eventType} in 'connected' state`)
              return state
          }
        case 'used':
          if (event.eventType === 'droneReady') {
            return {
              ...state,
              state: 'ready',
            }
          } else {
            console.log(`Never reach: ${event.eventType} in 'used' state`)
            return state
          }

        default:
          break
      }
      return state
    },
  }),

  all: (): Fish<Record<string, boolean>, DroneDefinedEvent> => ({
    fishId: FishId.of('com.awesome-home.drone.reg', 'all', 0),
    initialState: {},
    where: droneDefinedTag,
    onEvent: (state, event) => {
      state[event.id] = true
      return state
    },
  }),

  // Emitters
  emitDroneDefined,
  emitDroneReady,
  emitDroneConnected,
  emitDroneStatsUpdated,
  emitDroneLaunched,
  emitDroneStartedToNextWaypoint,
  emitDroneArrivedAtWaypoint,
  emitDroneMissionCompleted,
  emitDroneLanded,
  emitDroneDisconnected,
}
