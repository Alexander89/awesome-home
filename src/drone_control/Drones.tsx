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
  const [droneIp, setDroneIp] = React.useState('192.168.1.10')
  const pond = usePond()

  const drones = useRegistryFish(DroneTwins.all(), Object.keys, DroneTwins.of).map((f) => f.state)

  const registerDrone = () => {
    droneName && droneIp && DroneTwins.emitDroneReady(pond.emit, { id: droneName, ip: droneIp })
  }
  const droneReady = (id: string, ip: string) => () => {
    DroneTwins.emitDroneReady(pond.emit, { id, ip })
  }

  return (
    <Card style={{ margin: '0px 12px 12px 0px' }}>
      <CardHeader title={<Typography variant="h5">Drones</Typography>} />
      <CardContent>
        {drones.map((state) => (
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

            {state.state !== 'undefined' && state.state !== 'ready' && (
              <Button variant="contained" color="info" onClick={droneReady(state.id, state.ip)}>
                ready
              </Button>
            )}
          </Box>
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
        <Button variant="contained" onClick={registerDrone}>
          add
        </Button>
      </CardActions>
    </Card>
  )
}
