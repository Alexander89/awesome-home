import * as React from 'react'
import {
  Card,
  CardHeader,
  Typography,
  CardContent,
  Box,
  Button,
  CardActions,
  TextField,
} from '@material-ui/core'
import { usePond, useRegistryFish } from '@actyx-contrib/react-pond'
import { DroneTwins } from '../fish/DroneTwin'

type Props = {
  selectedDrone: string
  onSelectedDroneChanged: (name: string) => void
}

export const Drones = ({ selectedDrone, onSelectedDroneChanged }: Props): JSX.Element => {
  const [droneName, setDroneName] = React.useState('')
  const [droneIp, setDroneIp] = React.useState('192.168.10.1')
  const [droneSsid, setDroneSsid] = React.useState('TELLO-59FF95')
  const pond = usePond()

  const drones = useRegistryFish(DroneTwins.all(), Object.keys, DroneTwins.of).map((f) => f.state)

  const registerDrone = () => {
    droneName &&
      droneIp &&
      droneSsid &&
      DroneTwins.emitDroneDefined(pond.emit, { id: droneName, ip: droneIp, ssid: droneSsid })
  }
  const droneReady = (id: string) => () => {
    DroneTwins.emitDroneReady(pond.emit, { id })
  }

  return (
    <Card style={{ margin: '0px 12px 12px 0px' }}>
      <CardHeader title={<Typography variant="h5">Drones</Typography>} />
      <CardContent>
        {drones.map((state) => (
          <>
            <Box
              key={state.id}
              onClick={() => onSelectedDroneChanged(state.id)}
              style={{
                padding: 6,
                backgroundColor: selectedDrone === state.id ? '#C0C0C0' : 'white',
                display: 'flex',
                alignItems: 'center',
                borderRadius: 4,
                margin: 4,
                cursor: 'pointer',
                boxShadow:
                  selectedDrone === state.id
                    ? '0px 2px 4px -1px rgb(0 0 0 / 20%), 0px 4px 5px 0px rgb(0 0 0 / 14%), 0px 1px 10px 0px rgb(0 0 0 / 12%)'
                    : '',
              }}
            >
              <Box>
                {state.id} - {state.state}
              </Box>
              <Box style={{ flex: '1' }}>&nbsp;</Box>

              {state.state !== 'undefined' && (
                <Button variant="contained" color="info" onClick={droneReady(state.id)}>
                  ready
                </Button>
              )}
            </Box>
            <Box>
              <pre>{JSON.stringify(state, undefined, 2)}</pre>
            </Box>
          </>
        ))}
      </CardContent>

      <CardActions>
        <TextField
          id="drone-name"
          label="Name"
          value={droneName}
          onChange={({ target }) => setDroneName(target.value)}
        />
        <TextField
          id="drone-IP"
          value={droneIp}
          label="IP"
          onChange={({ target }) => setDroneIp(target.value)}
        />
      </CardActions>
      <CardActions>
        <TextField
          id="drone-SSID"
          value={droneSsid}
          label="SSID"
          onChange={({ target }) => setDroneSsid(target.value)}
        />
        <Button variant="contained" onClick={registerDrone}>
          add
        </Button>
      </CardActions>
    </Card>
  )
}
