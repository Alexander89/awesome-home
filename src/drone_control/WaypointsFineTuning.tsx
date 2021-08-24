import {
  Table,
  TableContainer,
  TableHead,
  Paper,
  TableBody,
  TableRow,
  TableCell,
  Button,
  TextField,
  ClickAwayListener,
  Typography,
  Box,
} from '@material-ui/core'
import * as React from 'react'
import './WaypointsFineTuning.css'
import { Waypoint, Waypoints } from '../fish/MissionTwin'

type Props = {
  waypoints: Waypoints
  setWaypoints: (wp: Waypoints) => void
  selectedWaypoint: number
  setSelectedWaypoint: (wpIdx: number) => void
}

const waypointId = (idx: number, wp: Waypoint): string => {
  switch (wp.type) {
    case 'goto':
      return `${idx}-goTo:${wp.mapX}-${wp.mapY}`
    case 'turn':
      return `${idx}-turn:${wp.deg}-${wp.duration}`
    case 'delay':
      return `${idx}-delay:${wp.duration}`
    default:
      return 'someOther'
  }
}

export const WaypointsFineTuning = ({
  waypoints,
  setWaypoints,
  selectedWaypoint,
  setSelectedWaypoint,
}: Props): JSX.Element => {
  const [modifyingField, setModifyingField] = React.useState<
    { idx: number; type: 'duration' | 'zValue' } | undefined
  >()
  const [modifyingValue, setModifyingValue] = React.useState<string>('')
  const inputRef = React.useRef<HTMLInputElement>(null)

  React.useEffect(() => {
    if (modifyingField) {
      const { idx, type } = modifyingField
      const wp = waypoints[idx]
      switch (type) {
        case 'duration':
          setModifyingValue(`${Math.round(wp.duration)}`)
          break
        case 'zValue':
          if (wp.type === 'goto') {
            setModifyingValue(`${wp.height}`)
          }
          break
      }
    }
  }, [modifyingField?.idx, modifyingField?.type])

  React.useEffect(() => {
    if (inputRef?.current) {
      inputRef.current.getElementsByTagName('input')[0]?.focus()
    }
  }, [inputRef?.current])

  const storeValue = (wp: Waypoint, keys: string | string[], value: number) => {
    if (!Array.isArray(keys)) {
      keys = [keys]
    } else {
      if (keys.length === 0) {
        return
      }
    }

    const last = keys.pop()!

    let ref: any = wp
    for (const k of keys) {
      ref = ref[k]
    }
    ref[last] = value

    // waypoints are modified by ref
    setWaypoints([...waypoints])
    setModifyingField(undefined)
  }

  const mkDestination = (wp: Waypoint, idx: number): string | JSX.Element | JSX.Element[] => {
    switch (wp.type) {
      case 'goto':
        const heightEdit = () => (
          <ClickAwayListener
            onClickAway={() => storeValue(wp, 'height', +modifyingValue)}
            mouseEvent="onMouseDown"
            touchEvent="onTouchStart"
          >
            <TextField
              type="number"
              ref={inputRef}
              inputProps={{ step: 10 }}
              value={+modifyingValue}
              style={{ maxWidth: 100 }}
              onChange={({ target }) => setModifyingValue(`${+target.value}`)}
              onBlur={(ev) => storeValue(wp, 'height', +ev.target.value)}
              onKeyDown={(ev: any) =>
                ev.code === 'enter' && storeValue(wp, 'height', +ev.target.value)
              }
            />
          </ClickAwayListener>
        )

        return (
          <Box>
            {idx !== 0 && <Typography>Distance: {wp.distance.toFixed(2)}m</Typography>}
            <Box style={{ display: 'flex', alignItems: 'center' }}>
              <Typography>Height: </Typography>
              {modifyingField && modifyingField.type === 'zValue' && modifyingField.idx === idx ? (
                heightEdit()
              ) : (
                <Typography onClick={() => changeHeight(idx)} className="changeable-value">
                  {wp.height.toFixed(2)}
                </Typography>
              )}
              <Typography>cm</Typography>
            </Box>
          </Box>
        )
      case 'turn':
        return <Typography>rotate: {wp.deg}°</Typography>
      case 'delay':
        return <Typography></Typography>
    }
  }
  const mkDuration = (wp: Waypoint, idx: number): string | JSX.Element => {
    if (modifyingField && modifyingField.type === 'duration' && modifyingField.idx === idx) {
      let step = 0.1
      let scale = 1000
      let unit = 'sec'
      if (wp.type === 'delay') {
        step = 100
        scale = 1
        unit = 'ms'
      }
      return (
        <ClickAwayListener
          onClickAway={() => storeValue(wp, 'duration', +modifyingValue)}
          mouseEvent="onMouseDown"
          touchEvent="onTouchStart"
        >
          <Box style={{ display: 'flex', alignItems: 'center' }}>
            <TextField
              type="number"
              ref={inputRef}
              inputProps={{ step, min: 0 }}
              value={+modifyingValue / scale}
              style={{ maxWidth: 100 }}
              onChange={({ target }) => setModifyingValue(`${+target.value * scale}`)}
              onBlur={(ev) => storeValue(wp, 'duration', +ev.target.value * scale)}
              onKeyDown={(ev: any) =>
                ev.code === 'enter' && storeValue(wp, 'duration', +ev.target.value * scale)
              }
            />
            <Typography>{unit}</Typography>
          </Box>
        </ClickAwayListener>
      )
    }
    switch (wp.type) {
      case 'goto':
      case 'turn':
        return (
          <Box style={{ display: 'flex' }}>
            <Typography onClick={() => changeDuration(idx)} className="changeable-value">
              {(wp.duration / 1000).toFixed(1)}
            </Typography>
            <Typography>sec</Typography>
          </Box>
        )
      case 'delay':
        return (
          <Box style={{ display: 'flex' }}>
            <Typography onClick={() => changeDuration(idx)} className="changeable-value">
              {wp.duration}
            </Typography>
            <Typography>ms</Typography>
          </Box>
        )
    }
  }
  const mkActions = (wp: Waypoint, idx: number): JSX.Element => {
    switch (wp.type) {
      case 'goto':
        return (
          <>
            <Button onClick={() => addDelay(idx)}>+ Delay</Button>
          </>
        )
      case 'turn':
        return <Button onClick={() => addDelay(idx)}>+ Delay</Button>
      case 'delay':
        return <Button onClick={() => removeWaypoint(idx)}>Remove</Button>
    }
  }

  const changeDuration = (idx: number) => setModifyingField({ type: 'duration', idx })
  const changeHeight = (idx: number) => setModifyingField({ type: 'zValue', idx })

  const addDelay = (idx: number) => {
    setWaypoints([
      ...waypoints.reduce<Waypoints>(
        (acc, wp, i) => [
          ...acc,
          wp,
          ...(i === idx ? ([{ type: 'delay', duration: 1000 }] as Waypoints) : []),
        ],
        [],
      ),
    ])
  }

  const removeWaypoint = (idx: number) => {
    setWaypoints([...waypoints.filter((_, i) => i !== idx)])
  }

  return (
    <TableContainer component={Paper}>
      <Table style={{ minWidth: 650 }} aria-label="simple table">
        <TableHead>
          <TableRow>
            <TableCell></TableCell>
            <TableCell>Type</TableCell>
            <TableCell width={250}>Target</TableCell>
            <TableCell width={140}>Duration</TableCell>
            <TableCell>Action</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {waypoints.map((row, idx) => (
            <TableRow
              key={waypointId(idx, row)}
              onClick={() => setSelectedWaypoint(idx)}
              style={{
                backgroundColor: selectedWaypoint === idx ? '#e4e6f8' : undefined,
                height: 90,
              }}
            >
              <TableCell component="th" scope="row">
                {idx}
              </TableCell>
              <TableCell>{idx === 0 ? 'take off' : row.type}</TableCell>
              <TableCell>{mkDestination(row, idx)}</TableCell>
              <TableCell>{mkDuration(row, idx)}</TableCell>
              <TableCell>{mkActions(row, idx)}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  )
}
