import { useFishFn } from '@actyx-contrib/react-pond'
import {
  Box,
  Button,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  TextField,
} from '@material-ui/core'
import * as React from 'react'
import { PlannerMode } from './MissionPlanner'
import { WaypointCoords } from './OpenLayerMap'
import { MissionTwins } from '../fish/MissionTwin'

type Props = {
  selectedMission: string | undefined
  onSelectedMissionChanged: (mission: string | undefined) => void
  mode: PlannerMode
  setMode: (mode: PlannerMode) => void
  track: WaypointCoords[]
  setTrack: (wp: WaypointCoords[]) => void
  onMissionDefined: (missionName: string, waypoints: WaypointCoords[]) => void
}

export const MapMenu = ({
  mode,
  setMode,
  track,
  onMissionDefined,
  selectedMission,
  onSelectedMissionChanged,
}: Props): JSX.Element => {
  const missions = Object.keys(useFishFn(MissionTwins.allVisible, 0)?.state || {})
  const [newMissionName, setNewMissionName] = React.useState('')

  const newMission = () => {
    setMode('new')
  }
  const removeMission = () => {}

  return (
    <Box style={{ display: 'flex' }}>
      {mode === 'idle' && (
        <>
          <Button variant="contained" title="new" onClick={newMission}>
            New
          </Button>
          <Button
            variant="contained"
            title="disable"
            onClick={removeMission}
            disabled={Boolean(selectedMission)}
          >
            disable
          </Button>
          <Box style={{ flex: '1' }}></Box>
          <FormControl variant="outlined" style={{ minWidth: 100 }}>
            <InputLabel id="mission-label">Mission</InputLabel>
            <Select
              labelId="mission-label"
              id="mission-select"
              value={selectedMission || ''}
              onChange={({ target }) =>
                onSelectedMissionChanged(Boolean(target.value) ? target.value : undefined)
              }
              label="Mission"
            >
              <MenuItem value=""></MenuItem>
              {missions.map((l) => (
                <MenuItem key={l} value={l}>
                  {l}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </>
      )}
      {(mode === 'new' || mode === 'newDefined') && (
        <>
          <Button variant="outlined" color="warning" title="New Track" onClick={newMission}>
            new track
          </Button>
          <TextField
            id="New Mission"
            label="Mission Name"
            value={newMissionName}
            onChange={({ target }) => setNewMissionName(target.value)}
          />
          {mode === 'newDefined' && (
            <Button
              variant="contained"
              title="Save"
              color="success"
              onClick={() => onMissionDefined(newMissionName, track)}
            >
              Save
            </Button>
          )}
        </>
      )}
    </Box>
  )
}
