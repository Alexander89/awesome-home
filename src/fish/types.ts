import { Tags } from '@actyx/pond'

type EventEmitter<ET> = <E>(tags: Tags<E>, event: E & ET) => any
export type Emitter<ET extends { eventType: string }> = <EE extends EventEmitter<ET>>(
  emit: EE,
  event: Omit<ET, 'eventType'> & Partial<Pick<ET, 'eventType'>>,
) => ReturnType<EE>

// export const mkEmitter =
//   <E extends { eventType: string }>(tag: Tags<E>, eventType: E['eventType']): Emitter<E> =>
//   (emit, event) =>
//     emit(tag, { eventType, ...event })
