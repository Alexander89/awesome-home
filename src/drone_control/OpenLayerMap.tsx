import React from 'react'
import { fromLonLat } from 'ol/proj'
import { platformModifierKeyOnly, shiftKeyOnly, doubleClick } from 'ol/events/condition'
import { LineString, Point } from 'ol/geom'
import 'ol/ol.css'

import monument from './img/placeholder.png'
import './OpenLayerMap.css'
import { RMap, ROSM, RInteraction, RLayerVector, RStyle, RFeature } from 'rlayers'
import { RView } from 'rlayers/RMap'
import { Vector as SourceVector } from 'ol/source'
import { RDrawProps } from 'rlayers/interaction/RDraw'
import { Box } from '@material-ui/core'

type Props = {
  missionId: string
}

type Res = {
  last: undefined | number[]
  rel: number[][]
}

export const OpenLayerMap = ({ missionId }: Props): JSX.Element => {
  const [selected, setSelected] = React.useState(false)
  const [view, setView] = React.useState<RView>({ center: fromLonLat([47.647, 11.342]), zoom: 11 })
  const [gpsPos, setGpsPos] = React.useState<Point | undefined>(undefined)

  const getLocation = () => {
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(({ coords }) => {
        const pos = fromLonLat([coords.longitude, coords.latitude])
        setView((v: RView) => ({ ...v, center: pos })), setGpsPos(new Point(pos))
      })
    }
  }
  React.useEffect(() => {
    getLocation()
  }, [])
  return (
    <Box>
      <Box style={{ height: 600 }}>
        <RMap className="example-map" initial={view} view={[view, setView]}>
          <ROSM />

          {gpsPos && (
            <RLayerVector>
              <RStyle.RStyle>
                <RStyle.RIcon src={monument} anchor={[0.5, 0.9]} />
              </RStyle.RStyle>
              <RFeature geometry={gpsPos} />
            </RLayerVector>
          )}

          {missionId && (
            <RLayerVector
              onChange={(e) => {
                // On every change, check if there is a feature covering the Eiffel Tower
                //@ts-ignore
                if (e.target?.forEachFeature) {
                  console.log(e.target)
                  const vs = e.target as SourceVector<LineString>
                  vs.forEachFeature((f) => {
                    const coords = f.getGeometry().getCoordinates()

                    const relCoords = coords.reduce<Res>(
                      (acc, c) => {
                        if (acc.last) {
                          const next = [c[0] - acc.last[0], c[1] - acc.last[1]]

                          return {
                            last: c,
                            rel: [...acc.rel, next],
                          }
                        } else {
                          return {
                            last: c,
                            rel: [[0, 0]],
                          }
                        }
                      },
                      { last: undefined, rel: [] },
                    ).rel
                    const dist = relCoords.map(([x, y]) => Math.sqrt(x ** 2 + y ** 2))
                    console.log(relCoords, dist)
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
                deleteCondition={React.useCallback(
                  (e) => platformModifierKeyOnly(e) && doubleClick(e),
                  [],
                )}
              />
            </RLayerVector>
          )}
        </RMap>
      </Box>
      <div>
        <p className="p-0 m-0">
          Hold <em>Shift</em> and click without dragging for a regular polygon
        </p>
        <p className="p-0 m-0">
          Hold <em>Shift</em> and <em>Alt</em> and drag for a freehand polygon
        </p>
        <p className="p-0 m-0">
          Hold <em>Alt</em> and click without dragging for a circle
        </p>
        <p className="p-0 m-0">
          Hold <em>Ctrl / &#x2318;</em> and drag to move/add a vertex
        </p>
        <p className="p-0 m-0">
          Hold <em>Ctrl / &#x2318;</em> and double click to remove a vertex
        </p>
      </div>
      <div className="mx-0 mt-1 mb-3 p-1 w-100 jumbotron shadow shadow">
        <p>Currently the Eiffel Tower is{selected ? '' : ' not'} covered</p>
      </div>
    </Box>
  )
}
