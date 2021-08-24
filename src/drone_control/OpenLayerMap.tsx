import React from 'react'
import { fromLonLat } from 'ol/proj'
import { platformModifierKeyOnly, shiftKeyOnly, doubleClick } from 'ol/events/condition'
import { LineString, Point } from 'ol/geom'
import { getDistance } from 'ol/sphere'
import 'ol/ol.css'

import monument from './img/placeholder.png'
import './OpenLayerMap.css'
import {
  RMap,
  ROSM,
  RInteraction,
  RLayerVector,
  RStyle,
  RFeature,
  RContext,
  RControl,
} from 'rlayers'
import { RView } from 'rlayers/RMap'
import { Vector as SourceVector } from 'ol/source'
import { RDrawProps } from 'rlayers/interaction/RDraw'

type Props = {
  mode: MapMode
}

type Res = {
  last: undefined | number[]
  rel: number[][]
}
export type MapMode =
  | {
      mode: 'view'
      track: WaypointCoords[]
      selected?: number
    }
  | {
      mode: 'new'
      onNewDefinedCompleted: (waypoints: WaypointCoords[]) => void
    }
  | {
      mode: 'edit'
    }

export const OpenLayerMap = ({ mode }: Props): JSX.Element => {
  const [view, setView] = React.useState<RView>({ center: fromLonLat([47.647, 11.342]), zoom: 11 })
  const [trackLocation, setTrackLocation] = React.useState(false)
  const [gpsPos, setGpsPos] = React.useState<Point | undefined>(undefined)

  const waypointLayer = React.useRef<RLayerVector>(null)

  const getLocation = () => {
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(({ coords }) => {
        const pos = fromLonLat([coords.longitude, coords.latitude])
        setView((v: RView) => ({ ...v, center: pos }))
        setGpsPos(new Point(pos))
      })
    }
  }
  React.useEffect(() => {
    getLocation()
  }, [])

  React.useEffect(() => {
    if (trackLocation && navigator.geolocation) {
      const tracker = setInterval(
        () =>
          navigator.geolocation.getCurrentPosition(({ coords }) => {
            setGpsPos(new Point(fromLonLat([coords.longitude, coords.latitude])))
          }),
        1000,
      )
      return () => {
        clearInterval(tracker)
      }
    }
    return () => {}
  }, [trackLocation])

  return (
    <RMap className="mission-map" noDefaultControls initial={view} view={[view, setView]}>
      <ROSM />
      <RControl.RScaleLine />
      <RControl.RAttribution />
      <RControl.RZoom />
      <RControl.RZoomSlider />
      <RControl.RCustom className="map-control map-control-track">
        <RContext.Consumer>
          {() => <button onClick={() => setTrackLocation((v) => !v)}>track</button>}
        </RContext.Consumer>
      </RControl.RCustom>

      <RControl.RCustom className="map-control map-control-set-wp">
        <RContext.Consumer>
          {() => (
            <button style={{ display: mode.mode === 'new' ? 'block' : 'none' }}>
              Set Waypoint
            </button>
          )}
        </RContext.Consumer>
      </RControl.RCustom>

      {gpsPos && (
        <RLayerVector>
          <RStyle.RStyle>
            <RStyle.RIcon src={monument} anchor={[0.5, 0.9]} />
          </RStyle.RStyle>
          <RFeature geometry={gpsPos} />
        </RLayerVector>
      )}

      {mode.mode === 'new' && (
        <RLayerVector
          ref={waypointLayer}
          onChange={function (e) {
            //@ts-ignore
            const layer = this as RLayerVector
            // On every change, check if there is a feature covering the Eiffel Tower
            //@ts-ignore
            if (e.target?.forEachFeature && waypointLayer.current?.context.map) {
              const vs = e.target as SourceVector<LineString>
              vs.forEachFeature((f) => {
                const coordsRaw = f.getGeometry().getCoordinates()
                const rawPoints = coordsRaw.map((v) => layer.context.map!.getPixelFromCoordinate(v))

                const [first, ...rest] = rawPoints
                const vecs = rest.reduce(
                  ({ last, acc }, pos) => ({
                    last: pos,
                    acc: [...acc, [pos[0] - last[0], pos[1] - last[1]]],
                  }),
                  { last: first, acc: [] as number[][] },
                ).acc

                const angles = mkAngles(vecs)
                const coords = (
                  f.getGeometry().clone().transform('EPSG:3857', 'EPSG:4326') as LineString
                ).getCoordinates()
                const waypoints = mapCoordsToWaypointCoords(coordsRaw, coords, angles)
                mode.onNewDefinedCompleted(waypoints)
              })
            }
          }}
        >
          {/* This is the style used for the drawn polygons */}
          <RStyle.RStyle>
            <RStyle.RStroke color="#0000ff" width={3} />
            <RStyle.RFill color="rgba(0, 0, 0, 0.75)" />
          </RStyle.RStyle>

          <RInteraction.RDraw
            type={'LineString' as RDrawProps['type']}
            freehandCondition={shiftKeyOnly}
          />

          <RInteraction.RModify
            condition={platformModifierKeyOnly}
            deleteCondition={(e) => platformModifierKeyOnly(e) && doubleClick(e)}
          />
        </RLayerVector>
      )}
      {mode.mode === 'view' && (
        <>
          <RLayerVector>
            {/* This is the style used for the drawn polygons */}
            <RStyle.RStyle>
              <RStyle.RStroke color="#0000ff" width={3} />
              <RStyle.RFill color="rgba(0, 0, 0, 0.75)" />
            </RStyle.RStyle>
            <RFeature geometry={new LineString(WaypointCoordsToMapCoords(mode.track))} />
          </RLayerVector>
          {mode.selected !== undefined && (
            <RLayerVector>
              {/* This is the style used for the drawn polygons */}
              <RStyle.RStyle>
                <RStyle.RIcon src={monument} anchor={[0.5, 0.9]} />
              </RStyle.RStyle>
              <RFeature
                geometry={new Point(WaypointCoordsToMapCoords([mode.track[mode.selected]])[0])}
              />
            </RLayerVector>
          )}
        </>
      )}
    </RMap>
  )
}

export type WaypointCoords = {
  mapX: number
  mapY: number
  angle: number
  dZ: number
  distance: number
}

export const mapCoordsToWaypointCoords = (
  rawCoords: number[][],
  coords: number[][],
  angles: number[],
): WaypointCoords[] => {
  const [first, ...rest] = coords
  const mapToLength = rest.reduce(
    ({ last, acc }, v) => ({
      last: v,
      acc: [...acc, [last, v]],
    }),
    { last: first, acc: [] as number[][][] },
  ).acc

  const distances = mapToLength.reduce<number[]>(
    (acc, [from, to]) => [...acc, getDistance(from, to)],
    [0],
  )

  return distances.reduce<WaypointCoords[]>(
    (acc, distance, i) => [
      ...acc,
      {
        mapX: rawCoords[i][0],
        mapY: rawCoords[i][1],
        angle: angles[i],
        dZ: 0,
        distance,
      },
    ],
    [],
  )
}

const mkAngles = (vects: number[][]): number[] => {
  const toDegScaler = 180 / Math.PI
  const toDeg = (rad: number): number => rad * toDegScaler

  const angle = (v: number[]): number => toDeg(Math.atan2(v[0], v[1]))

  const [first, ...rawAngles] = vects.map(angle)
  return rawAngles
    .reduce(({ acc, last }, a) => ({ acc: [...acc, last - a], last: a }), {
      acc: [0],
      last: first,
    })
    .acc.map((a) => (a > 180 ? a - 360 : a < -180 ? a + 360 : a))
}

export const WaypointCoordsToMapCoords = (wp: WaypointCoords[]): [number, number][] =>
  wp.reduce<[number, number][]>((acc, p) => [...acc, [p.mapX, p.mapY]], [])
