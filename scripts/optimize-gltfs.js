import { DenoIO } from 'https://esm.sh/@gltf-transform/core'
import { KHRONOS_EXTENSIONS } from 'https://esm.sh/@gltf-transform/extensions'
import {
  resample,
  prune,
  dedup,
  textureResize,
} from 'https://esm.sh/@gltf-transform/functions@3.0.4'
import * as path from 'https://deno.land/std/path/mod.ts'
import { getNestedFileByName } from './util.js'

const io = new DenoIO().registerExtensions(KHRONOS_EXTENSIONS)

const gltfDirectory = path.joinGlobs([Deno.cwd(), 'assets', 'gltf'])
for (const gltfPath of await getNestedFileByName(gltfDirectory, (entryPath) =>
  entryPath.endsWith('.glb')
)) {
  const glbBytes = await Deno.readFile(gltfPath)
  const glbDocument = await io.readBinary(glbBytes)

  await glbDocument.transform(resample(), dedup())

  const newGlbBytes = await io.writeBinary(glbDocument)
  await Deno.writeFile(gltfPath, newGlbBytes)
}
