import {
  Box,
  Button,
  Card,
  CardContent,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  Typography,
} from '@material-ui/core'
import * as React from 'react'
import { OpenLayerMap } from './OpenLayerMap'

type Props = {
  selectedMission: string | undefined
  onSelectedMissionChanged: (mission: string) => void
}

export const MissionPlanner = ({
  selectedMission,
  onSelectedMissionChanged,
}: Props): JSX.Element => {
  return (
    <Card style={{ height: '100%' }}>
      <Box style={{ display: 'flex' }}>
        <Button title="new" onClick={newMission} />
        <Button title="disable" onClick={removeMission} disable={Boolean(selectedMission)} />

        <Box style={{ display: 'flex' }}>
          <Typography variant="h5">Drone launch-pad</Typography>
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
              <MenuItem value="">
                <em>None</em>
              </MenuItem>
              {missions.map((l) => (
                <MenuItem key={l} value={l}>
                  {l}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </Box>
      </Box>
      <CardContent style={{ height: '100%' }}>
        <OpenLayerMap missionId="1" />
      </CardContent>
    </Card>
  )
}
