import { Fish, FishId, Tag } from '@actyx/pond'
import { Emitter } from './types'
import { DroneMissionCompletedEvent, DroneTwins } from './DroneTwin'

export type UndefinedState = {
  state: 'undefined'
  id: string
}
export type ReadyState = {
  state: 'ready'
  drone: string
  id: string
}
export type ActivatedState = {
  state: 'activated'
  drone: string
  id: string
}
export type UsedState = {
  state: 'used'
  id: string
  missionId: string
}
export type LaunchPadState = UndefinedState | ReadyState | ActivatedState | UsedState

export type MissionLogState = {
  nextMissions: Array<String>
  currentMission?: {
    assignedDrone: string
    id: string
  }
  completedMissions: Array<{
    ts: Date
    missionId: string
    drone: string
  }>
}

export type LaunchPadRegisteredEvent = {
  eventType: 'launchPadRegistered'
  id: string
}
export type DroneMountedEvent = {
  eventType: 'droneMounted'
  id: string
  drone: string
}
export type DroneActivatedEvent = {
  eventType: 'droneActivated'
  id: string
  drone: string
}
export type ActivateDroneTimeoutEvent = {
  eventType: 'activateDroneTimeout'
  id: string
  drone: string
}
export type DroneStartedEvent = {
  eventType: 'droneStarted'
  id: string
  drone: string
  missionId: string
}

export type MissionQueuedEvent = {
  eventType: 'missionQueued'
  missionId: string
  launchpadId: string
}

export type LaunchPadEvent =
  | DroneMountedEvent
  | DroneActivatedEvent
  | ActivateDroneTimeoutEvent
  | DroneStartedEvent
  | LaunchPadRegisteredEvent
  | MissionQueuedEvent

type MissionLogEvent = DroneStartedEvent | MissionQueuedEvent | DroneMissionCompletedEvent

const emitLaunchPadRegistered: Emitter<LaunchPadRegisteredEvent> = (emit, event) =>
  emit(launchpadTag.withId(event.id).and(launchpadRegisteredTag), {
    eventType: 'launchPadRegistered',
    ...event,
  })

const emitDroneMounted: Emitter<DroneMountedEvent> = (emit, event) =>
  emit(launchpadTag.withId(event.id), {
    eventType: 'droneMounted',
    ...event,
  })

const emitDroneActivated: Emitter<DroneActivatedEvent> = (emit, event) =>
  emit(launchpadTag.withId(event.id), { eventType: 'droneActivated', ...event })

const emitActivateDroneTimeout: Emitter<ActivateDroneTimeoutEvent> = (emit, event) =>
  emit(launchpadTag.withId(event.id), { eventType: 'activateDroneTimeout', ...event })

const emitDroneStarted: Emitter<DroneStartedEvent> = (emit, event) =>
  emit(launchpadTag.withId(event.id).and(launchpadLaunchTag), {
    eventType: 'droneStarted',
    ...event,
  })

const emitMissionCreated: Emitter<MissionQueuedEvent> = (emit, event) =>
  emit(launchpadTag.withId(event.launchpadId).and(missionQueuedTag), {
    eventType: 'missionQueued',
    ...event,
  })

const launchpadTag = Tag<LaunchPadEvent>('launchpad')
const launchpadLaunchTag = Tag<DroneStartedEvent>('launchpad.launch')
const launchpadRegisteredTag = Tag<LaunchPadRegisteredEvent>('launchpad.registered')
const missionQueuedTag = Tag<MissionQueuedEvent>('mission.queued')

export const LaunchPadTwins = {
  // Tags
  tags: {
    launchpadTag,
  },

  // Twins
  of: (id: string): Fish<LaunchPadState, LaunchPadEvent> => ({
    fishId: FishId.of('com.awesome-home.launchpad', id, 0),
    initialState: { state: 'undefined', id },
    where: launchpadTag.withId(id),
    onEvent: (state, event) => {
      switch (event.eventType) {
        case 'droneMounted':
          return {
            id,
            drone: event.drone,
            state: 'ready',
          }
        case 'droneActivated':
          return {
            id,
            drone: event.drone,
            state: 'activated',
          }
        case 'droneStarted':
          return {
            id,
            state: 'used',
            missionId: event.missionId,
          }
        case 'activateDroneTimeout':
          if (state.state !== 'used') {
            return {
              id,
              drone: event.drone,
              state: 'ready',
            }
          }
      }
      return state
    },
  }),

  all: (): Fish<Record<string, boolean>, LaunchPadRegisteredEvent> => ({
    fishId: FishId.of('com.awesome-home.launchpad.reg', 'all', 0),
    initialState: {},
    where: launchpadRegisteredTag,
    onEvent: (state, event) => {
      state[event.id] = true
      return state
    },
  }),

  missionLog: (id: string): Fish<MissionLogState, MissionLogEvent> => ({
    fishId: FishId.of('com.awesome-home.launchpad.missionLog', id, 0),
    initialState: {
      nextMissions: [],
      currentMission: undefined,
      completedMissions: [],
    },
    where: missionQueuedTag
      .or(launchpadTag.withId(id).and(launchpadLaunchTag))
      .or(DroneTwins.tags.droneMissionCompletedTag),
    onEvent: (state, event, { timestampAsDate }) => {
      switch (event.eventType) {
        case 'missionQueued':
          if (event.launchpadId === id) {
            state.nextMissions.push(event.missionId)
          }
          return state
        case 'droneStarted':
          if (event.id === id) {
            const nextIdx = state.nextMissions.findIndex((s) => s === event.missionId)
            if (nextIdx === -1) {
              console.log('start an unknown mission')
            } else {
              delete state.nextMissions[nextIdx]
            }

            state.currentMission = {
              assignedDrone: event.drone,
              id: event.missionId,
            }
          }
          return state
        case 'droneMissionCompleted':
          state.nextMissions = state.nextMissions.filter((s) => s !== event.missionId)
          console.log(state.currentMission, event.missionId)
          if (state.currentMission?.id === event.missionId) {
            state.currentMission = undefined
          }

          state.completedMissions.push({
            drone: event.id,
            missionId: event.missionId,
            ts: timestampAsDate(),
          })
          return state

        default:
          break
      }
      return state
    },
  }),

  // Emitters
  emitDroneMounted,
  emitDroneActivated,
  emitActivateDroneTimeout,
  emitDroneStarted,
  emitLaunchPadRegistered,
  emitMissionCreated,
}
