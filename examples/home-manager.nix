{ ... }:
{
  # adjust this path, or import the flake's homeManagerModules.default.
  imports = [ ../nix/home-manager.nix ];

  programs.waybar-dnd = {
    enable = true;
    bars = [ "laptop" "monitor" ];
  };

  programs.waybar.settings = {
    laptop.modules-center = [ "clock" "custom/dnd" ];
    monitor.modules-center = [ "clock" "custom/dnd" ];
  };
}
