---
name: drawio
description: Always use when user asks to create, generate, draw, or design a diagram, flowchart, architecture diagram, ER diagram, sequence diagram, class diagram, network diagram, mockup, wireframe, or UI sketch, or mentions draw.io, drawio, drawoi, .drawio files, or diagram export to PNG/SVG/PDF.
---

# Draw.io Diagram Skill

Generate draw.io diagrams as native `.drawio` files. Optionally export to PNG, SVG, or PDF with the diagram XML embedded (so the exported file remains editable in draw.io), or generate a browser URL that opens the diagram directly in the draw.io editor.

## Установка MCP (Важно!)
Чтобы этот скилл работал максимально эффективно в IDE и агентах, необходимо добавить `@drawio/mcp` в файл конфигурации MCP `~/.gemini/config/mcp.json`:
```json
{
  "mcpServers": {
    "drawio": {
      "command": "npx",
      "args": ["-y", "@drawio/mcp"]
    }
  }
}
```
После добавления необходимо **перезапустить агента** (или MCP daemon).

## How to create a diagram

1. **Generate draw.io XML** in mxGraphModel format for the requested diagram
2. **Write the XML** to a `.drawio` file in the current working directory using the Write tool
3. **Handle the requested output format**:
   - `png` / `svg` / `pdf` → locate the draw.io CLI, export with `--embed-diagram`, then delete the source `.drawio` file. If the CLI is not found, keep the `.drawio` file and tell the user they can install the draw.io desktop app to enable export, or use `url` mode instead, or open the `.drawio` file directly
   - `url` → generate a browser URL from the XML and open it (see Browser URL output). Keep the `.drawio` file as a persistent local copy
   - *(no format)* → no extra step; the `.drawio` file is the output
4. **Open the result** — the exported file if exported, the browser URL if `url`, or the `.drawio` file otherwise. If the open command fails, print the file path (or URL) so the user can open it manually

## Choosing the output format

Check the user's request for a format preference. Examples:

- `/drawio create a flowchart` → `flowchart.drawio`
- `/drawio png flowchart for login` → `login-flow.drawio.png`
- `/drawio svg: ER diagram` → `er-diagram.drawio.svg`
- `/drawio pdf architecture overview` → `architecture-overview.drawio.pdf`
- `/drawio url flowchart for user login` → opens browser at `app.diagrams.net`

If no format is mentioned, just write the `.drawio` file and open it in draw.io. The user can always ask to export later.

## Browser URL output

When the user requests `url` format, generate a draw.io URL that opens the diagram directly in the browser editor at `app.diagrams.net` — no draw.io Desktop required.

### URL generation

Run this `node -e` one-liner to read the `.drawio` file and print the URL (replace `DIAGRAM.drawio` with the actual filename):

```bash
URL=$(node -e '
const fs = require("fs");
const zlib = require("zlib");
const xml = fs.readFileSync(process.argv[1], "utf8");
const compressed = zlib.deflateRawSync(encodeURIComponent(xml)).toString("base64");
const payload = encodeURIComponent(JSON.stringify({ type: "xml", compressed: true, data: compressed }));
console.log("https://app.diagrams.net/?grid=0&pv=0&border=10&edit=_blank#create=" + payload);
' DIAGRAM.drawio)
```
