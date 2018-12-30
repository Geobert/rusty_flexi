# rusty_flexi
Flexi time tracker in Rust

Lucky to work in a company where Flexi time is available, I take the opportunity to write
a flexi time tracker to learn Rust.

# Quick start
At first launch, use arrows key to select and configure your schedule and how many
holidays you have, and how much you have left for the current year and hit ESC to begin.
Press 'o' to recall this screen (will apply to not already created month).

On the main view, use UP/DOWN to select the day to edit and ENTER to go to edit mode. Use
LEFT/RIGHT to select the field, UP/DOWN to edit it by one unit, or type the wanted value.
Press ENTER or ESC to finish.

Press HOME to focus the current day.

# Full hotkey list
## Main view mode
 * `Home` to go to today
 * `Enter` to toggle edit mode
 * `Esc` to exit edit mode/options
 * `b` to set beginning by current time
 * `e` to set end by current time
 * `h` to toggle holiday
 * `s` to toggle sick day
 * `Arrow Up/Down` navigate by day
 * `Arrow Left/Right` navigate by week
 * `Page Up/Down` to navigate by month
 * `o` to open options

## Edit mode
 * `Arrows left/right` to move fields
 * `Arrows up/down` to change field by one increment
 * Typing number edit the field as well

# Tricks
You can manually add to `settings.json` just before the last closing brace:
```
"offsets": {
  "entry": value_in_minutes,
  "exit": value_in_minutes,
}
```
This will affect `b` (entry's value) and `e` (exit's value) hotkeys by removing/adding
this offset to the current time.

I use it to set the time it takes to go from the badge reader to my desk so the input time
is correct.

# Build instruction
The whole project is only tested on Windows 7/10, should build on other platform with a
few edits but I'm not able to provide support if any issue arise.

On Windows, to use `build.bat`, you need http://angusj.com/resourcehacker/ to be installed
in order to have an icon with the exe.
Edit `build.bat` according to the installation path.

If you don't care about the icon, just run `cargo build --release`. 
