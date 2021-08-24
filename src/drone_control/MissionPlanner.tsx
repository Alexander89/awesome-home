import { Box, Card, CardContent } from '@material-ui/core'
import * as React from 'react'
import { GoToWaypoint, MissionTwins, Waypoints } from '../fish/MissionTwin'
import { MapMode, OpenLayerMap, WaypointCoords } from './OpenLayerMap'
import { MapMenu } from './MapMenu'
import { WaypointsFineTuning } from './WaypointsFineTuning'
import uuid from 'uuid'
import { usePond } from '@actyx-contrib/react-pond'
import { CancelSubscription } from '@actyx/pond'

export type PlannerMode = 'idle' | 'new' | 'newDefined' | 'view' | 'edit'

export const MissionPlanner = (): JSX.Element => {
  const [selectedMission, setSelectedMission] = React.useState<string | undefined>(undefined)

  const [mode, setMode] = React.useState<PlannerMode>('idle')
  const [track, setTrack] = React.useState<WaypointCoords[]>([])
  const [selectedWaypoint, setSelectedWaypoint] = React.useState<number | undefined>(0)
  const [waypoints, setWaypoints] = React.useState<Waypoints>([])
  const pond = usePond()

  const saveMission = (name: string) => {
    setMode('idle')
    MissionTwins.emitDefineMission(pond.emit, {
      id: uuid.v4(),
      name,
      waypoints,
    })
  }
  const onCancel = () => {
    setMode('idle')
    setWaypoints([])
    setTrack([])
    setSelectedWaypoint(undefined)
  }
  const onDelete = () => {
    if (selectedMission) {
      MissionTwins.emitShowMission(pond.emit, {
        id: selectedMission,
        visible: false,
      })
      onCancel()
    }
  }

  React.useEffect(() => {
    console.log('subscribe to mission:', selectedMission)
    if (selectedMission) {
      let cancel: CancelSubscription | undefined = pond.observe(
        MissionTwins.of(selectedMission),
        (state) => {
          cancel && cancel()
          cancel = undefined
          if (state.defined) {
            const track = waypointsToWaypointCoords(state.waypoints)
            setWaypoints(state.waypoints)
            setTrack(track)
            setMode('newDefined')

            console.log('newDefined', track, waypoints)
          }
        },
      )
      return () => {
        cancel && cancel()
      }
    }
    return () => {}
  }, [selectedMission])

  const toMapMode = React.useCallback((): MapMode => {
    switch (mode) {
      case 'new':
        return {
          mode: 'new',
          onNewDefinedCompleted: (waypoints: WaypointCoords[]) => {
            setTrack(waypoints)
            setWaypoints(waypointCoordsToWaypoints(waypoints))
            setMode('newDefined')
          },
        }
      default:
      case 'idle':
        return {
          mode: 'view',
          track: [],
        }
      case 'view':
      case 'newDefined':
        return {
          mode: 'view',
          track: track,
          selected:
            selectedWaypoint !== undefined
              ? countRealWaypoints(waypoints, selectedWaypoint)
              : undefined,
        }
    }
  }, [mode, track, selectedWaypoint])

  return (
    <Card style={{ height: '100%' }}>
      <CardContent style={{ height: '100%' }}>
        <MapMenu
          selectedMission={selectedMission}
          onSelectedMissionChanged={setSelectedMission}
          mode={mode}
          setMode={setMode}
          onSaveNewMission={saveMission}
          onCancel={onCancel}
          onDelete={onDelete}
        />
        {mode === 'newDefined' && (
          <Box style={{ maxHeight: 500, overflow: 'auto' }}>
            <WaypointsFineTuning
              waypoints={waypoints}
              setWaypoints={setWaypoints}
              selectedWaypoint={selectedWaypoint || 0}
              setSelectedWaypoint={setSelectedWaypoint}
            />
          </Box>
        )}
        <Box style={{ height: 600 }}>
          <OpenLayerMap mode={toMapMode()} />
        </Box>
        <Box>
          <p className="p-0 m-0">
            Hold <em>Shift</em> and click without dragging for a regular polygon
          </p>
          <p className="p-0 m-0">
            Hold <em>Shift</em> and <em>Alt</em> and drag for a freehand polygon
          </p>
        </Box>
      </CardContent>
    </Card>
  )
}

const waypointCoordsToWaypoints = (coords: WaypointCoords[]): Waypoints => {
  if (coords.length === 0) {
    return []
  }

  return coords.reduce<Waypoints>((acc, { mapX, mapY, height, angle, distance }) => {
    return [
      ...acc,
      {
        type: 'goto',
        mapX,
        mapY,
        height,
        angle,
        distance,
        duration: 2000 + distance * 5000,
      },
      ...(angle > 1 || angle < -1
        ? ([
            {
              type: 'turn',
              deg: Math.round(angle),
              duration: 4_000,
            },
          ] as Waypoints)
        : ([] as Waypoints)),
    ]
  }, [])
}

const countRealWaypoints = (wp: Waypoints, idx: number): number =>
  Math.max(
    wp.reduce<number>((acc, w, i) => (w.type === 'goto' && i < idx ? acc + 1 : acc), -1),
    0,
  )
const waypointsToWaypointCoords = (wp: Waypoints): WaypointCoords[] =>
  wp
    .filter((wp): wp is GoToWaypoint => wp.type === 'goto')
    .map<WaypointCoords>(({ mapX, mapY, angle, height, distance }) => ({
      mapX,
      mapY,
      angle,
      height,
      distance,
    }))
