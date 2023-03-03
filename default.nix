with import <nixos> {};

stdenv.mkDerivation rec {
  name = "bevy";
  buildInputs = [
    gcc xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr libGL freetype pkgconfig
    freetype.dev expat alsa-lib udev sqlite libxkbcommon wayland gtk3 gtk3-x11 cairo harfbuzz glib
  ];

  LD_LIBRARY_PATH = builtins.foldl'
    (a: b: "${a}:${b}/lib") "${vulkan-loader}/lib" buildInputs;
}
