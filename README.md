# ani-gif

Simple tool for converting gifs into .ani files.

### Example

Create a new curser from `cursor.gif`.
```
ani-gif convert -g .\cursor.gif -a cursor.ani -f 5 --hotspot 0:0
```
- `-g`: path to gif file.
- `-a`: path to ani file.
- The `-f` argument dictates how many 1/60s a frame stays on screen.
- `--hotspot` takes an argument that defines where the hotspot of the cursor is. `0:0` is the default value and is the upper left corner.