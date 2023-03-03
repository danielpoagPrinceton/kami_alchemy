let 
  pkgs = import <unstable> {};

  pkgs-mingw = import <nixpkgs> { crossSystem = {config = "x86_64-w64-mingw32"; }; };
in 
  pkgs-mingw.mkShell {
    nativeBuildInputs = [
      pkgs.gcc pkgs.xorg.libX11 pkgs.xorg.libXcursor pkgs.xorg.libXi pkgs.xorg.libXrandr pkgs.libGL pkgs.freetype
      pkgs.pkgconfig pkgs.freetype.dev pkgs.expat pkgs.alsa-lib pkgs.udev pkgs.sqlite pkgs.libxkbcommon pkgs.wayland
    ];
    buildInputs = [
      pkgs-mingw.sqlite
      pkgs-mingw.windows.pthreads pkgs-mingw.windows.mingw_w64_pthreads
    ]; 
}
