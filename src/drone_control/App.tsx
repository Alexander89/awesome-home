import * as React from 'react'
import { Box, Container } from '@material-ui/core'
import { LaunchPad } from './LaunchPad'
import { Drones } from './Drones'
import { MissionPlanner } from './MissionPlanner'

export const App = (): JSX.Element => {
  const [selectedDrone, setSelectedDrone] = React.useState('')
  const [selectedLaunchpad, setSelectedLaunchpad] = React.useState<string | undefined>(undefined)

  return (
    <Container style={{ display: 'flex' }}>
      <Box style={{ flex: '0 0 300px' }}>
        <LaunchPad
          selectedLaunchpad={selectedLaunchpad}
          onSelectionChanged={setSelectedLaunchpad}
          selectedDrone={selectedDrone}
        />
        <Drones selectedDrone={selectedDrone} onSelectedDroneChanged={setSelectedDrone} />
      </Box>

      <Box style={{ flex: '1' }}>
        <MissionPlanner />
      </Box>
    </Container>
  )
}
