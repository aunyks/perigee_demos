async function getNestedFileByName(searchPath, searchParams) {
  let names = []

  for await (const dirEntry of Deno.readDir(searchPath)) {
    const entryPath = `${searchPath}/${dirEntry.name}`
    if (dirEntry.isDirectory) {
      names.push(await getNestedFileByName(entryPath, searchParams))
    } else {
      if (searchParams(entryPath)) {
        names.push(entryPath)
      }
    }
  }

  return names.flat(1)
}

export { getNestedFileByName }
