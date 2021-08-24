import { Box, Card, CardContent } from '@material-ui/core'
import * as React from 'react'
import { TurnWaypoint, Waypoints } from '../fish/MissionTwin'
import { MapMode, OpenLayerMap, WaypointCoords } from './OpenLayerMap'

import { MapMenu } from './MapMenu'
import { WaypointsFineTuning } from './WaypointsFineTuning'

type Props = {
  selectedMission: string | undefined
  onSelectedMissionChanged: (mission: string | undefined) => void
}
export type PlannerMode = 'idle' | 'new' | 'newDefined' | 'view' | 'edit'

const waypointCoordsToWaypoints = (coords: WaypointCoords[]): Waypoints => {
  if (coords.length === 0) {
    return []
  }

  return coords.reduce<Waypoints>((acc, { mapX, mapY, dZ, angle, distance }) => {
    return [
      ...acc,
      {
        type: 'goto',
        mapX,
        mapY,
        dZ,
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

export const MissionPlanner = ({
  selectedMission,
  onSelectedMissionChanged,
}: Props): JSX.Element => {
  const [mode, setMode] = React.useState<PlannerMode>('idle')
  const [track, setTrack] = React.useState<WaypointCoords[]>([])
  const [newMissionName, setNewMissionName] = React.useState<string>('')
  const [selectedWaypoint, setSelectedWaypoint] = React.useState<number | undefined>(0)
  const [waypoints, setWaypoints] = React.useState<Waypoints>([])

  const missionDefined = (name: string, waypoints: WaypointCoords[]) => {
    setMode('idle')
    setTrack(waypoints)
    setNewMissionName(name)
    setWaypoints(waypointCoordsToWaypoints(waypoints))
    // MissionTwins.emitDefineMission(pond.emit, {
    //   id: uuid.v4(),
    //   name,
    //   waypoints,
    // })
  }

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
          mode={mode}
          setMode={setMode}
          track={track}
          setTrack={setTrack}
          onMissionDefined={missionDefined}
          selectedMission={selectedMission}
          onSelectedMissionChanged={onSelectedMissionChanged}
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
