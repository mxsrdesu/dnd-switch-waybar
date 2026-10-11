# dnd-switch-waybar

a dnd switch for waybar, using dunst.

## install

requires linux, waybar, dunst and rust 1.89 or newer.

```sh
cargo install --path . --locked
```

add `~/.cargo/bin` and `dunstctl` to waybar's path, or use the full binary path.
merge [the module](examples/waybar.jsonc) into your waybar config and add
`custom/dnd` to the panel's module list.

for nix, run `nix build` or use the [home manager example](examples/home-manager.nix).
the nix package includes `dunstctl`. the module doesn't start or configure dunst.
remove any existing `custom/dnd` commands when switching to the home manager module.

## usage

left-click toggles dunst's notification pause. the panel shows `DND: on` or
`DND: off` and checks the state every five seconds. panels use the same dunst.

`waybar-dnd status` prints json. `waybar-dnd toggle` switches the pause state.
if reading dunst fails, the panel keeps showing `DND: off` and the error goes
to stderr. that fallback doesn't confirm that notifications are enabled.
a failed toggle returns a nonzero exit code.

## style

use your waybar config for formatting and your external stylesheet for colours,
background, spacing and borders: `#custom-dnd.on` and `#custom-dnd.off`.

## prepare an existing panel

```sh
cargo build --locked --release
python3 scripts/prepare-waybar.py --config /path/to/active-waybar.json \
  --binary ~/.local/bin/waybar-dnd --output integration
```

this writes a preview and diff. it only replaces existing dnd commands;
it doesn't install the binary, edit your active config or restart waybar.

[mit license](LICENSE).
