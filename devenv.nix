{ pkgs, lib, config, inputs, ... }:

{
  languages.rust = {
    enable = true;
    channel = "stable";
  };
}
