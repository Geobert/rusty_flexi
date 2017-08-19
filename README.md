# rusty_flexi
Flexi time tracker in Rust

Lucky to work in a company where Flexi time is available, I take the opportunity to write a flexi time tracker to learn Rust.

At first launch, use arrows key to select and configure your schedule and how many holidays you have, and how much you have left for the current year and hit ESC to begin. Press 'o' to recall this screen (will apply to not already created month).

On the main view, use UP/DOWN to select the day to edit and ENTER to go to edit mode. 
Use LEFT/RIGHT to select the field, UP/DOWN to edit it by one unit, or type the wanted value. Press ENTER or ESC to finish.

Press HOME to focus the current day.

## Build instruction
On Windows, to use `build.bat`, you need http://angusj.com/resourcehacker/ to be installed in order to have an icon with the exe.
Edit `build.bat` according to the installation path.

If you don't care about the icon, just `cargo build --release`. 
