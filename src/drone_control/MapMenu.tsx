import { useRegistryFish } from '@actyx-contrib/react-pond'
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
import { DefinedState, MissionTwins } from '../fish/MissionTwin'

type Props = {
  selectedMission: string | undefined
  onSelectedMissionChanged: (mission: string | undefined) => void
  mode: PlannerMode
  setMode: (mode: PlannerMode) => void
  onSaveNewMission: (missionName: string) => void
  onCancel: () => void
  onDelete: () => void
}

export const MapMenu = ({
  selectedMission,
  onSelectedMissionChanged,
  mode,
  setMode,
  onSaveNewMission,
  onCancel,
  onDelete,
}: Props): JSX.Element => {
  const missions = useRegistryFish(MissionTwins.allVisible(), Object.keys, MissionTwins.of)
    .map((s) => s.state)
    .filter((s): s is DefinedState => s.defined)
    .map(({ name, id }) => ({ name, id }))

  const [newMissionName, setNewMissionName] = React.useState('')

  const newMission = () => {
    setMode('new')
  }

  return (
    <Box style={{ display: 'flex' }}>
      {mode === 'idle' && (
        <>
          <Button variant="contained" title="new" onClick={newMission}>
            New
          </Button>
          <Box style={{ flex: '1' }}></Box>
          <FormControl variant="outlined" style={{ minWidth: 100 }}>
            <InputLabel id="mission-label">Mission</InputLabel>
            <Select
              labelId="mission-label"
              id="mission-select"
              value={selectedMission || ''}
              onChange={({ target }) => {
                if (Boolean(target.value)) {
                  const [id, name] = target.value.split('::')
                  setNewMissionName(name)
                  onSelectedMissionChanged(id)
                } else {
                  onSelectedMissionChanged(undefined)
                }
              }}
              label="Mission"
            >
              <MenuItem value=""></MenuItem>
              {missions.map(({ name, id }) => (
                <MenuItem key={id} value={id + '::' + name}>
                  {name}
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
            <>
              <Button
                variant="contained"
                title="Save"
                color="success"
                onClick={() => onSaveNewMission(newMissionName)}
              >
                Save
              </Button>
              <Button variant="contained" title="Delete" color="error" onClick={() => onDelete()}>
                Delete
              </Button>
            </>
          )}
          <Button variant="contained" title="Cancel" color="warning" onClick={() => onCancel()}>
            Cancel
          </Button>
        </>
      )}
    </Box>
  )
}
