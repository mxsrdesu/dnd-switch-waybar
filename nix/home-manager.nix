{ config, lib, pkgs, ... }:
let
  cfg = config.programs.waybar-dnd;
  moduleSettings = lib.mapAttrsRecursive (_: lib.mkDefault)
    ((builtins.fromJSON (builtins.readFile ../examples/waybar.jsonc))."custom/dnd" // {
      exec = "${cfg.package}/bin/waybar-dnd status";
      on-click = "${cfg.package}/bin/waybar-dnd toggle";
    });
in {
  options.programs.waybar-dnd = {
    enable = lib.mkEnableOption "dnd switch for waybar";
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../package.nix {};
      description = "the waybar-dnd package";
    };
    bars = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "waybar settings names to add the custom module to; set modules-center separately";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    programs.waybar.settings = lib.genAttrs cfg.bars (_: {
      "custom/dnd" = moduleSettings;
    });
  };
}
