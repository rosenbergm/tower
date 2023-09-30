import * as D from 'schemawax'

const portDecoder = D.object({
  required: {
    "PrivatePort": D.number,
  },
  optional: {
    "IP": D.nullable(D.string),
    "PublicPort": D.nullable(D.number)
  }
})

const creatingContainer = D.object({
  required: {
    status: D.literal('creating'),
  }
})

const runningContainer = D.object({
  required: {
    status: D.literal('running'),
    image_url: D.nullable(D.string),
    exposing_port: D.nullable(D.array(portDecoder)),
  }
})

const error = D.object({
  required: {
    status: D.literal('error'),
  }
})

const other = D.object({
  required: {
    status: D.literal('other'),
  }
})

const containerDetailsDecoder = D.oneOf(creatingContainer, runningContainer, error, other)

export const dockerAppDecoder = D.object({
  required: {
    name: D.string,
    domain: D.string,
    container_details: containerDetailsDecoder,
  }
})

export const dockerAppsDecoder = D.array(dockerAppDecoder);

export type DockerAppT = D.Output<typeof dockerAppDecoder>

const socketMessageDecoder = D.oneOf(
  D.literal('creating'),
  D.literal('error'),
  D.literal('refreshing'),
  D.literal('started'),
  D.literal('upgrading')
)

export const wsDecoder = D.object({
  required: {
    app_name: D.string,
    msg: socketMessageDecoder
  }
})

const staticAppDecoder = D.object({
  required: {
    name: D.string,
    domain: D.string,
    mountpoint: D.nullable(D.string),
    entrypoint: D.nullable(D.string),
  }
})

export type StaticAppT = D.Output<typeof staticAppDecoder>;

export const staticAppsDecoder = D.array(staticAppDecoder);