# Swaysome

This binary helps you configure sway to work a bit more like Awesome. This
currently means workspaces that are name-spaced on a per-screen basis.

It should also work with i3, but this is untested.


## Usage

Build and install the `swaysome` binary somewhere in your `$PATH` with something
like:
```
git clone https://git.hya.sk/skia/swaysome
cd swaysome
cargo install --path .
```

Then create the file (and the directory if needed) "~/.config/sway/config.d/swaysome.conf" and paste this inside:
```
# Change focus between workspaces
unbindsym $mod+1
unbindsym $mod+2
unbindsym $mod+3
unbindsym $mod+4
unbindsym $mod+5
unbindsym $mod+6
unbindsym $mod+7
unbindsym $mod+8
unbindsym $mod+9
unbindsym $mod+0
bindsym $mod+1 exec "swaysome focus 1"
bindsym $mod+2 exec "swaysome focus 2"
bindsym $mod+3 exec "swaysome focus 3"
bindsym $mod+4 exec "swaysome focus 4"
bindsym $mod+5 exec "swaysome focus 5"
bindsym $mod+6 exec "swaysome focus 6"
bindsym $mod+7 exec "swaysome focus 7"
bindsym $mod+8 exec "swaysome focus 8"
bindsym $mod+9 exec "swaysome focus 9"
bindsym $mod+0 exec "swaysome focus 0"

# Move containers between workspaces
unbindsym $mod+Shift+1
unbindsym $mod+Shift+2
unbindsym $mod+Shift+3
unbindsym $mod+Shift+4
unbindsym $mod+Shift+5
unbindsym $mod+Shift+6
unbindsym $mod+Shift+7
unbindsym $mod+Shift+8
unbindsym $mod+Shift+9
unbindsym $mod+Shift+0
bindsym $mod+Shift+1 exec "swaysome move 1"
bindsym $mod+Shift+2 exec "swaysome move 2"
bindsym $mod+Shift+3 exec "swaysome move 3"
bindsym $mod+Shift+4 exec "swaysome move 4"
bindsym $mod+Shift+5 exec "swaysome move 5"
bindsym $mod+Shift+6 exec "swaysome move 6"
bindsym $mod+Shift+7 exec "swaysome move 7"
bindsym $mod+Shift+8 exec "swaysome move 8"
bindsym $mod+Shift+9 exec "swaysome move 9"
bindsym $mod+Shift+0 exec "swaysome move 0"

# Move focused container to next output
bindsym $mod+o exec "swaysome next_output"

# Move focused container to previous output
bindsym $mod+Shift+o exec "swaysome prev_output"

# Init workspaces for every screen
exec "swaysome init 1"
```

Finally append your `sway` configuration with this:
```
include ~/.config/sway/config.d/*.conf
```

You should end-up with workspaces from `1` to `0`, prefixed with a screen index,
giving you workspace `01` on the first screen, and workspace `11` on the second
one, both accessible with shortcut `$mod+1`.

The `init` command simply walks through every screen to initialize a prefixed
workspace. It does it backwards so that you end-up focused on the first screen,
as usual.


## Exhaustive swaysome commands list

* `move [name]`: move the focused container to `[name]`
* `next_output`: move the focused container to the next output
* `prev_output`: move the focused container to the previous output
* `focus [name]`: change focus to `[name]`
* `focus_all_outputs [name]`: change all outputs focus to `[name]`
* `init [name]`: cycle all outputs to create a default workspace with name `[name]`

