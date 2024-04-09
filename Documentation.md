# Documentation

## Enhanced backgrounds (BKG)

The BKG or "Background" mod allows backgrounds to be modularlly configured using json files without needing to recompile the source.

Please see [this page](https://wiki.doukutsu.club/bkg-hack) for more info on BKG in general, as well as other implementations for other Cave Story engines.

The BKG mod adds the following commands to TSC:

- `<BKGname_of_config$` - Loads the BKG config file from `./data/bkg` with `name_of_config`. (`$` is string delimiter)
- `<BKDwwww` - Disable the backgroubd layer `wwww`. (out-of-range layers will be set to the last layer)
- `<BKEwwww` - Enable background layer. (simmilar to `BKD`)
- `<BKPwwww:xxxx:yyyy` - Set `BKG` parameter `xxxx` for layer `wwww` to value `yyyy`. *(TODO: negatives and floating points)*
- `<BKR` - Restores background to default parameters for the map, simmilar to a `TRA` command.


### BKP assignment table
This table is lifted from `componets/background.rs`
```
//parameter is xxxx
//value is yyyy

match parameter
{
    0 => layer_ref.layer_enabled = value != 0,
    1 => layer_ref.bmp_x_offset = value,
    2 => layer_ref.bmp_y_offset = value,
    3 => layer_ref.bmp_width = value,
    4 => layer_ref.bmp_height = value,
    5 => layer_ref.draw_repeat_x = value,
    6 => layer_ref.draw_repeat_y = value,
    7 => layer_ref.draw_repeat_gap_x = value,
    8 => layer_ref.draw_repeat_gap_y = value,
    9 => layer_ref.draw_corner_offset_x = value as f32,
    10 => layer_ref.draw_corner_offset_y = value as f32,

    //animation_style
    11 => layer_ref.animation_style.frame_count = value,
    12 => layer_ref.animation_style.frame_start = value,
    13 => layer_ref.animation_style.animation_speed = value,
    14 => layer_ref.animation_style.follow_speed_x = value as f32,
    15 => layer_ref.animation_style.follow_speed_y = value as f32,
    16 => layer_ref.animation_style.autoscroll_speed_x = value as f32,
    17 => layer_ref.animation_style.autoscroll_speed_y = value as f32,

    //scroll flags (set from bitfield)
    18 => {
        layer_ref.animation_style.scroll_flags.follow_pc_x = 0 < (value & 1 << 0);
        layer_ref.animation_style.scroll_flags.follow_pc_y = 0 < (value & 1 << 1);
        layer_ref.animation_style.scroll_flags.autoscroll_x = 0 < (value & 1 << 2);
        layer_ref.animation_style.scroll_flags.autoscroll_y = 0 < (value & 1 << 3);
        layer_ref.animation_style.scroll_flags.align_with_water_lvl = 0 < (value & 1 << 4);
        layer_ref.animation_style.scroll_flags.draw_above_foreground = 0 < (value & 1 << 5);
        layer_ref.animation_style.scroll_flags.random_offset_x = 0 < (value & 1 << 6);
        layer_ref.animation_style.scroll_flags.random_offset_y = 0 < (value & 1 << 7);
        layer_ref.animation_style.scroll_flags.lock_to_x_axis = 0 < (value & 1 << 8);
        layer_ref.animation_style.scroll_flags.lock_to_y_axis = 0 < (value & 1 << 9);
        layer_ref.animation_style.scroll_flags.randomize_all_parameters = 0 < (value & 1 << 10);
    }
    //invalid parameter: do nothing
    _ => {}
}
```

When setting scroll flags from a `BKP` command, they are set using a bitfield, simmilar to how items are equipped on the player with the vanilla command `EQ+`. Raise 2 to the power of the shift count for the parameter seen in the code above and add all parameter numbers together to set the flags.

Example:
```
Flag: align with water is value 2^4=16
Flag: follow_pc_x is value 2^0=1
Flag: autoscroll_y = 2^3=8

Total: 25

Setting all these flags to TRUE and all others to FALSE will use the value 0025.

This is the command to set these flags on layer 0:
<BKP0000:0018:0025

```

### Config File Layout

To save the effort of running multiple TSC commands for each background config, all configuration data is stored in a json file in the `./data/bkg` directory.
These files can have any name, and the bitmap they load is determined by the fields within the file.

- `version` - config version number, this should not be edited by the user, since it tells the engine how to deal with the config file.
- `bmp_filename` - what image file to load from the `./data` directory (Note: the image file does *not* have to exclusively be a bitmap: d-rs can handle .png and .pbm as well. Either of these filetypes will work just the same, despite the bmp naming convention used here)
- `lighting_mode` - what type of ambient lighting d-rs uses Types are:
    - None (0),
    - BackgroundOnly (1),
    - Ambient (2),
- `layers` - a list of each layer to be drawn on the background, the first entry in the list will be drawn at the back, and the other entries will work their way up from there.
- `bmp_x_offset` /  `bmp_y_offset` - the top left corner on the background image where the rect to draw for this layer is
- `bmp_width` / `bmp_height` - how wide and tall the rect to draw should be
- `draw_repeat_x` / `draw_repeat_y` - how many times to repeat the bitmap, starting from the top left corner and extending right and down. To repeat a definite count in the other direction, apply a negative value to the `draw_corner_offset` fields. If these are set to 0, they will repeat infinitely in both directions (left/right or up/down).
- `draw_repeat_gap_x` / `draw_repeat_gap_y` how much space to have between bitmap "tiles", in pixels.
- `draw_corner_offset_x` / `draw_corner_offset_y` - offset from the top left corner where the tiles should begin drawing
- `animation_style` sub-group that controls how the background moves and changes
- `frame_count` - how many frames of animation the layer should have, 0 and 1 both do the same thing
- `frame_start` - what animation number to start on, [0 thru frame_count-1]
- `animation_speed` - how many ingame ticks it takes to advance the frame by 1
- `follow_speed_x` / `follow_speed_y` - how fast the background follows the camera's motion. For it to move with the foreground, set these values to `1.0`. For the classic "slow follow" effect, set them to `0.5`. They can also be negative. These will only be active if the flags `follow_pc_x` / `follow_pc_y` are set to `true`.
- `autoscroll_speed_x` / `autoscroll_speed_y` - same as `follow_speed` parameters, but apply to the autoscrolling. Units are speed * 1 pixel per second.
- `scroll_flags` - determines what properties to apply to the scrolling, the parameters of which are outlined above.
- `follow_pc_x` / `follow_pc_y` - the background will move as a multiple of the the user's camera.
- `autoscroll_x` / `autoscroll_y` - the background will move as a multiple of time
- `align_with_water_lvl` - will follow the global water level, like what's used in the Core fight
- `draw_above_foreground` - draws the background above the frontmost tile layer
- `random_offset_x` / `random_offset_y` - only applies to the opposite autoscroll mode with a finite number of `draw_repeat`. Each time the last rect in the chain goes offscreen, it will loop back on the other side. If this field is set to `true` it will have a randomized `x`/`y` value. This is good for things like clouds, where you'd set it to autoscroll along the x axis, but enable the `random_offset_y` flag to have each "cloud" come back onscreen with a randomized height. The ammount that this value will deviate each time is between `-animation_speed` and `animation_speed`.
- `lock_to_x_axis` / `lock_to_y_axis` locks the map's background to either the X or Y axis by compounding x1 frame movement onto the current offset. This same effect can be achieved with other settings, but this can be combined with other settings to create different effects.

Here's an example of a file:
```
{
  "version": 1,
  "bmp_filename": "bkBlue",
  "lighting_mode": 0,
  "layers": [
    {
      "layer_enabled": true,
      "bmp_x_offset": 0,
      "bmp_y_offset": 0,
      "bmp_width": 64,
      "bmp_height": 64,
      "draw_repeat_x": 0,
      "draw_repeat_y": 0,
      "draw_repeat_gap_x": 0,
      "draw_repeat_gap_y": 0,
      "draw_corner_offset_x": 0.0,
      "draw_corner_offset_y": 0.0,
      "animation_style": {
        "frame_count": 0,
        "frame_start": 0,
        "animation_speed": 0,
        "follow_speed_x": 1.0,
        "follow_speed_y": 1.0,
        "autoscroll_speed_x": 1.0,
        "autoscroll_speed_y": 1.0,
        "scroll_flags": {
          "follow_pc_x": false,
          "follow_pc_y": false,
          "autoscroll_x": false,
          "autoscroll_y": false,
          "align_with_water_lvl": false,
          "draw_above_foreground": false,
          "random_offset_x": false,
          "random_offset_y": false,
          "lock_to_x_axis": false,
          "lock_to_y_axis": false,
          "randomize_all_parameters": false
        }
      }
    },
    {
      "layer_enabled": true,
      "bmp_x_offset": 0,
      "bmp_y_offset": 0,
      "bmp_width": 64,
      "bmp_height": 64,
      "draw_repeat_x": 0,
      "draw_repeat_y": 0,
      "draw_repeat_gap_x": 16,
      "draw_repeat_gap_y": 16,
      "draw_corner_offset_x": 600.0,
      "draw_corner_offset_y": 600.0,
      "animation_style": {
        "frame_count": 0,
        "frame_start": 0,
        "animation_speed": 60,
        "follow_speed_x": 1.0,
        "follow_speed_y": 1.0,
        "autoscroll_speed_x": -0.5,
        "autoscroll_speed_y": 0.5,
        "scroll_flags": {
          "follow_pc_x": false,
          "follow_pc_y": true,
          "autoscroll_x": true,
          "autoscroll_y": true,
          "align_with_water_lvl": false,
          "draw_above_foreground": false,
          "random_offset_x": false,
          "random_offset_y": false,
          "lock_to_x_axis": false,
          "lock_to_y_axis": false,
          "randomize_all_parameters": false
        }
      }
    }
  ]
}
```
(to try this out ingame, name it something like `background_name.json` and place it in the `/data/bkg` directory), then load it in with `<BKGbackground_name$`


## Layers

Layers mode adds the ability to draw four distinct maps' worth of tiles directly on top of each other, simplifying tileest bitmaps and allowing for some interesting layouts. It also extends the builtin tileset limit so the tilesets themselves can be much larger.

Please see [this page](https://wiki.doukutsu.club/layers-mode) for a more in-depth explaination on how the layer mod works.

In addition to support for layers, the following commands have been added to interface with them:


- `<CMLwwww:xxxx:yyyy:zzzz` - Sets the tile at (xxxx,yyyy) to type zzzz, on layer wwww [0/back, 1/mid, 2/fore, 3/far fore]
- `<SMLwwww:xxxx:yyyy` - Subtracts 1 from tile type at (xxxx,yyyy) on layer wwww [0/back, 1/mid, 2/fore, 3/far fore]



