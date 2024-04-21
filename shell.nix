{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell rec {
	nativeBuildInputs = with pkgs; [
		rustup
		pkg-config
		wayland
		libxkbcommon
		xorg.libX11
		xorg.libXcursor
		xorg.libXrandr
		xorg.libXi
		libglvnd
		libsForQt5.kdialog
	];
	LD_LIBRARY_PATH="${pkgs.libglvnd}/lib:${pkgs.libxkbcommon}/lib:${pkgs.xorg.libX11}/lib";
}
