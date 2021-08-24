import * as React from 'react'
import { useFishFn, usePond } from '@actyx-contrib/react-pond'
import {
  Card,
  CardHeader,
  Box,
  Typography,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  CardContent,
  CardActions,
  Button,
  TextField,
} from '@material-ui/core'
import { LaunchPadTwins } from '../fish/LaunchpadTwin'
import { DefinedState, MissionTwins } from '../fish/MissionTwin'
import { useRegistryFish } from '@actyx-contrib/react-pond'

type Props = {
  onSelectionChanged: (id: string | undefined) => void
  selectedLaunchpad: string | undefined
  selectedDrone: string | undefined
}

export const LaunchPad = ({
  onSelectionChanged,
  selectedLaunchpad,
  selectedDrone,
}: Props): JSX.Element => {
  const [launchpadName, setLaunchpadName] = React.useState('')
  const [selectedMission, setSelectedMission] = React.useState<string | undefined>(undefined)

  const pond = usePond()

  const launchpads = Object.keys(useFishFn(LaunchPadTwins.all, 0)?.state || {})
  const launchpad = useFishFn(LaunchPadTwins.of, selectedLaunchpad)
  const launchpadMissionLog = useFishFn(LaunchPadTwins.missionLog, selectedLaunchpad)

  const missions = useRegistryFish(MissionTwins.allVisible(), Object.keys, MissionTwins.of).map(
    (f) => f.state,
  )
  const availableMissions = missions.filter((m): m is DefinedState => m.defined)

  const onStartMission = () => {
    selectedLaunchpad &&
      selectedMission &&
      LaunchPadTwins.emitMissionCreated(pond.emit, {
        launchpadId: selectedLaunchpad,
        missionId: selectedMission,
      })
  }

  const droneMounted = () => {
    selectedLaunchpad &&
      selectedDrone &&
      LaunchPadTwins.emitDroneMounted(pond.emit, {
        drone: selectedDrone,
        id: selectedLaunchpad,
      })
  }
  const registerLaunchpad = () => {
    launchpadName && LaunchPadTwins.emitLaunchPadRegistered(pond.emit, { id: launchpadName })
  }

  return (
    <Card style={{ margin: '0px 12px 12px 0px' }}>
      <CardHeader
        title={
          <Box style={{ display: 'flex' }}>
            <Typography variant="h5">Launchpad</Typography>
            <Box style={{ flex: '1' }}></Box>
            <FormControl variant="outlined" style={{ minWidth: 100 }}>
              <InputLabel id="launchpad-label">Launchpad</InputLabel>
              <Select
                labelId="launchpad-label"
                id="launchpad-select"
                value={selectedLaunchpad || ''}
                onChange={({ target }) =>
                  onSelectionChanged(Boolean(target.value) ? target.value : undefined)
                }
                label="Launchpad"
              >
                <MenuItem value="">
                  <em>None</em>
                </MenuItem>
                {launchpads.map((l) => (
                  <MenuItem key={l} value={l}>
                    {l}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>
        }
      />
      {selectedLaunchpad && (
        <>
          <CardContent>
            {!!launchpad && (
              <Box style={{ display: 'flex' }}>
                <Box>{launchpad.state.id}</Box>
                <Box style={{ flex: 1 }}></Box>
                <Box>{launchpad.state.state}</Box>
              </Box>
            )}
            {!!launchpadMissionLog && (
              <Box>
                <Box>
                  <Box>Next missions</Box>
                  <Box>{launchpadMissionLog.state.nextMissions.join(', ')}</Box>
                </Box>
                {launchpadMissionLog.state.currentMission && (
                  <Box>
                    <Box>Current mission</Box>
                    <Box>{launchpadMissionLog.state.currentMission.id}</Box>
                    <Box>{launchpadMissionLog.state.currentMission.assignedDrone}</Box>
                  </Box>
                )}
                <Box>
                  <Box>Completed missions</Box>
                  <Box>
                    {launchpadMissionLog.state.completedMissions
                      .map((m) => `${m.ts.toLocaleTimeString()} : ${m.missionId} - ${m.drone}`)
                      .join(', ')}
                  </Box>
                </Box>
              </Box>
            )}
          </CardContent>
          <CardActions>
            <FormControl variant="outlined" style={{ minWidth: 100 }}>
              <InputLabel id="Mission-label">Mission</InputLabel>
              <Select
                labelId="Mission-label"
                id="Mission-select"
                value={selectedMission || ''}
                onChange={({ target }) =>
                  setSelectedMission(Boolean(target.value) ? target.value : undefined)
                }
                label="Mission"
              >
                <MenuItem value="">
                  <em></em>
                </MenuItem>
                {availableMissions.map((m) => (
                  <MenuItem key={m.id} value={m.id}>
                    {m.name} -{' '}
                    {(m.waypoints.reduce((acc, v) => acc + v.duration, 0) / 1000).toFixed(1)} sec
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
            <Button variant="contained" onClick={onStartMission}>
              Start mission
            </Button>
          </CardActions>
          <CardActions>
            <Button variant="outlined" onClick={droneMounted}>
              Drone Mounted
            </Button>
          </CardActions>
        </>
      )}
      <CardActions>
        <TextField
          label="Name"
          value={launchpadName}
          onChange={({ target }) => setLaunchpadName(target.value)}
        />
        <Button variant="contained" onClick={registerLaunchpad}>
          Register Launchpad
        </Button>
      </CardActions>
    </Card>
  )
}
