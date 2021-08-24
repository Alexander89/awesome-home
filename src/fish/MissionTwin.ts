import { Fish, FishId, Tag } from '@actyx/pond'
import { Emitter } from './types'

export type GoToWaypoint = {
  type: 'goto'
  mapX: number
  mapY: number
  height: number
  angle: number
  distance: number
  duration: number
}

export type TurnWaypoint = {
  type: 'turn'
  deg: number
  duration: number
}

export type DelayWaypoint = {
  type: 'delay'
  duration: number
}

export type Waypoint = GoToWaypoint | TurnWaypoint | DelayWaypoint
export type Waypoints = Array<Waypoint>

export type UndefinedState = {
  defined: false
  id: string
}
export type DefinedState = {
  defined: true
  id: string
  name: string
  visible: boolean
  waypoints: Waypoints
}
export type MissionState = UndefinedState | DefinedState

/*
defineMission(
  id: string,
  waypoints: Waypoints,
)
showMission(
  id: string,
  visible: boolean,
)
 */

export type DefineMissionEvent = {
  eventType: 'defineMission'
  id: string
  name: string
  waypoints: Waypoints
}
export type ShowMissionEvent = {
  eventType: 'showMission'
  id: string
  visible: boolean
}
export type MissionEvent = DefineMissionEvent | ShowMissionEvent

const emitDefineMission: Emitter<DefineMissionEvent> = (emit, event) =>
  emit(missionTag.withId(event.id), { eventType: 'defineMission', ...event })

const emitShowMission: Emitter<ShowMissionEvent> = (emit, event) =>
  emit(missionTag.withId(event.id), { eventType: 'showMission', ...event })

const missionTag = Tag<MissionEvent>('mission')
const missionOrderTag = Tag<MissionEvent>('mission.order')
export const MissionTwins = {
  // Tags
  tags: {
    missionTag,
    missionOrderTag,
  },
  // Twins
  of: (id: string): Fish<MissionState, MissionEvent> => ({
    fishId: FishId.of('com.awesome-home.mission', id, 0),
    initialState: { defined: false, id },
    where: missionTag.withId(id),
    onEvent: (state, event) => {
      switch (event.eventType) {
        case 'defineMission':
          return {
            defined: true,
            id,
            name: event.name,
            visible: state.defined ? state.visible : true,
            waypoints: event.waypoints,
          }
        case 'showMission':
          if (state.defined) {
            state.visible = event.visible
            return state
          } else {
            return {
              defined: true,
              id,
              name: id,
              visible: event.visible,
              waypoints: [],
            }
          }
      }
      return state
    },
  }),

  all: (): Fish<Record<string, boolean>, MissionEvent> => ({
    fishId: FishId.of('com.awesome-home.mission.reg', 'all', 0),
    initialState: {},
    where: missionTag,
    onEvent: (state, event) => {
      state[event.id] = true
      return state
    },
  }),
  allVisible: (): Fish<Record<string, boolean>, MissionEvent> => ({
    fishId: FishId.of('com.awesome-home.mission.reg', 'allVisible', 0),
    initialState: {},
    where: missionTag,
    onEvent: (state, event) => {
      if (event.eventType === 'showMission' && event.visible === false) {
        delete state[event.id]
      } else {
        state[event.id] = true
      }
      console.log(event, state)
      return state
    },
  }),

  // Emitters
  emitDefineMission,
  emitShowMission,
}
