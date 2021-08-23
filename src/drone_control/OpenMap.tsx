import * as React from 'react'
import { Map, View } from 'ol'
import TileLayer from 'ol/layer/Tile'
import XYZ from 'ol/source/XYZ'
import Stamen from 'ol/source/Stamen'

export type MapLayers = 'satellite' | 'vector' | 'watercolor'

type Props = {
  mapLayer: MapLayers
}

export const OpenMap = ({ mapLayer }: Props): JSX.Element => {
  const mapRef = React.useRef<HTMLDivElement>(null)
  const [map, setMap] = React.useState<Map | undefined>(undefined)

  React.useEffect(() => {
    if (mapRef.current) {
      const localMap = new Map({
        target: mapRef.current.id,
        layers: [],
        view: new View({
          center: [0, 0],
          zoom: 2,
        }),
      })
      setMap(localMap)
      setLayer(localMap, mapLayer)

      return () => {
        localMap.dispose()
        setMap(undefined)
      }
    } else {
      return () => {}
    }
  }, [mapRef.current])

  const setLayer = (map: Map, layerType: MapLayers) => {
    const l = map.getLayers()
    console.log(l)
    switch (layerType) {
      case 'satellite':
        map.addLayer(
          new TileLayer({
            source: new XYZ({
              url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
            }),
          }),
        )
        break
      case 'vector':
        map.addLayer(
          new TileLayer({
            source: new XYZ({
              url: 'https://{a-c}.tile.openstreetmap.org/{z}/{x}/{y}.png',
            }),
          }),
        )
        break
      case 'watercolor':
        map.addLayer(
          new TileLayer({
            source: new Stamen({
              layer: 'watercolor',
            }),
          }),
        )
        break
      default:
        break
    }
    console.log(map.getLayers())
  }
  return <div id="map" ref={mapRef} style={{ height: '100%' }}></div>
}
